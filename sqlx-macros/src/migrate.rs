use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use sha2::{Digest, Sha384};
use sqlx_core::migrate::MigrationType;
use std::fs;
use std::path::Path;
use syn::LitStr;

pub struct QuotedMigrationType(MigrationType);

impl ToTokens for QuotedMigrationType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ts = match self.0 {
            MigrationType::Simple => quote! { ::sqlx::migrate::MigrationType::Simple },
            MigrationType::ReversibleUp => quote! { ::sqlx::migrate::MigrationType::ReversibleUp },
            MigrationType::ReversibleDown => {
                quote! { ::sqlx::migrate::MigrationType::ReversibleDown }
            }
        };
        tokens.append_all(ts.into_iter());
    }
}

struct QuotedMigration {
    version: i64,
    description: String,
    migration_type: QuotedMigrationType,
    path: String,
    checksum: Vec<u8>,
}

impl ToTokens for QuotedMigration {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let QuotedMigration {
            version,
            description,
            migration_type,
            path,
            checksum,
        } = &self;

        let ts = quote! {
            ::sqlx::migrate::Migration {
                version: #version,
                description: ::std::borrow::Cow::Borrowed(#description),
                migration_type:  #migration_type,
                // this tells the compiler to watch this path for changes
                sql: ::std::borrow::Cow::Borrowed(include_str!(#path)),
                checksum: ::std::borrow::Cow::Borrowed(&[
                    #(#checksum),*
                ]),
            }
        };

        tokens.append_all(ts.into_iter());
    }
}

// mostly copied from sqlx-core/src/migrate/source.rs
pub(crate) fn expand_migrator_from_lit_dir(dir: LitStr) -> crate::Result<TokenStream> {
    expand_migrator_from_dir(&dir.value(), dir.span())
}

pub(crate) fn expand_migrator_from_dir(
    dir: &str,
    err_span: proc_macro2::Span,
) -> crate::Result<TokenStream> {
    let path = crate::common::resolve_path(dir, err_span)?;

    expand_migrator(&path)
}

pub(crate) fn expand_migrator(path: &Path) -> crate::Result<TokenStream> {
    let mut migrations = Vec::new();

    for entry in fs::read_dir(&path)? {
        let entry = entry?;
        if !fs::metadata(entry.path())?.is_file() {
            // not a file; ignore
            continue;
        }

        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();

        let parts = file_name.splitn(2, '_').collect::<Vec<_>>();

        if parts.len() != 2 || !parts[1].ends_with(".sql") {
            // not of the format: <VERSION>_<DESCRIPTION>.sql; ignore
            continue;
        }

        let version: i64 = parts[0].parse()?;

        let migration_type = MigrationType::from_filename(parts[1]);
        // remove the `.sql` and replace `_` with ` `
        let description = parts[1]
            .trim_end_matches(migration_type.suffix())
            .replace('_', " ")
            .to_owned();

        let sql = fs::read_to_string(&entry.path())?;

        let checksum = Vec::from(Sha384::digest(sql.as_bytes()).as_slice());

        // canonicalize the path so we can pass it to `include_str!()`
        let path = entry.path().canonicalize()?;
        let path = path
            .to_str()
            .ok_or_else(|| {
                format!(
                    "migration path cannot be represented as a string: {:?}",
                    path
                )
            })?
            .to_owned();

        migrations.push(QuotedMigration {
            version,
            description,
            migration_type: QuotedMigrationType(migration_type),
            path,
            checksum,
        })
    }

    // ensure that we are sorted by `VERSION ASC`
    migrations.sort_by_key(|m| m.version);

    #[cfg(any(sqlx_macros_unstable, procmacro2_semver_exempt))]
    {
        let path = path.canonicalize()?;
        let path = path.to_str().ok_or_else(|| {
            format!(
                "migration directory path cannot be represented as a string: {:?}",
                path
            )
        })?;

        proc_macro::tracked_path::path(path);
    }

    Ok(quote! {
        ::sqlx::migrate::Migrator {
            migrations: ::std::borrow::Cow::Borrowed(&[
                #(#migrations),*
            ]),
            ignore_missing: false,
            locking: true,
        }
    })
}
