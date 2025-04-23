#[cfg(any(sqlx_macros_unstable, procmacro2_semver_exempt))]
extern crate proc_macro;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::LitStr;

use sqlx_core::migrate::{Migration, MigrationType};

pub struct QuoteMigrationType(MigrationType);

impl ToTokens for QuoteMigrationType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ts = match self.0 {
            MigrationType::Simple => quote! { ::sqlx::migrate::MigrationType::Simple },
            MigrationType::ReversibleUp => quote! { ::sqlx::migrate::MigrationType::ReversibleUp },
            MigrationType::ReversibleDown => {
                quote! { ::sqlx::migrate::MigrationType::ReversibleDown }
            }
        };
        tokens.append_all(ts);
    }
}

struct QuoteMigration {
    migration: Migration,
    path: PathBuf,
}

impl ToTokens for QuoteMigration {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Migration {
            version,
            description,
            migration_type,
            checksum,
            no_tx,
            ..
        } = &self.migration;

        let migration_type = QuoteMigrationType(*migration_type);

        let sql = self
            .path
            .canonicalize()
            .map_err(|e| {
                format!(
                    "error canonicalizing migration path {}: {e}",
                    self.path.display()
                )
            })
            .and_then(|path| {
                let path_str = path.to_str().ok_or_else(|| {
                    format!(
                        "migration path cannot be represented as a string: {}",
                        self.path.display()
                    )
                })?;

                // this tells the compiler to watch this path for changes
                Ok(quote! { include_str!(#path_str) })
            })
            .unwrap_or_else(|e| quote! { compile_error!(#e) });

        let ts = quote! {
            ::sqlx::migrate::Migration {
                version: #version,
                description: ::std::borrow::Cow::Borrowed(#description),
                migration_type:  #migration_type,
                sql: ::std::borrow::Cow::Borrowed(#sql),
                no_tx: #no_tx,
                checksum: ::std::borrow::Cow::Borrowed(&[
                    #(#checksum),*
                ]),
            }
        };

        tokens.append_all(ts);
    }
}

pub fn expand_migrator_from_lit_dir(
    dir: LitStr,
    parameters: Option<HashMap<String, String>>,
) -> crate::Result<TokenStream> {
    expand_migrator_from_dir(&dir.value(), dir.span(), parameters)
}

pub(crate) fn expand_migrator_from_dir(
    dir: &str,
    err_span: proc_macro2::Span,
    parameters: Option<HashMap<String, String>>,
) -> crate::Result<TokenStream> {
    let path = crate::common::resolve_path(dir, err_span)?;
    expand_migrator(&path, parameters)
}

pub(crate) fn expand_migrator(
    path: &Path,
    parameters: Option<HashMap<String, String>>,
) -> crate::Result<TokenStream> {
    const ENABLE_SUBSTITUTION: &str = "-- enable-substitution";
    const DISABLE_SUBSTITUTION: &str = "-- disable-substitution";
    let path = path.canonicalize().map_err(|e| {
        format!(
            "error canonicalizing migration directory {}: {e}",
            path.display()
        )
    })?;

    // Use the same code path to resolve migrations at compile time and runtime.
    let mut substitution_enabled = false;
    let migrations = sqlx_core::migrate::resolve_blocking(&path)?
        .into_iter()
        .map(|(migration, path)| {
            if let Some(ref params) = parameters {
                for line in migration.sql.lines() {
                    let trimmed_line = line.trim();
                    if trimmed_line == ENABLE_SUBSTITUTION {
                        substitution_enabled = true;
                        continue;
                    } else if trimmed_line == DISABLE_SUBSTITUTION {
                        substitution_enabled = false;
                        continue;
                    }
                    if substitution_enabled {
                        subst::substitute(line, params).expect("Missing substitution parameter");
                    } 
                }
            }
            QuoteMigration { migration, path }
        });

    #[cfg(any(sqlx_macros_unstable, procmacro2_semver_exempt))]
    {
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
            ..::sqlx::migrate::Migrator::DEFAULT
        }
    })
}
