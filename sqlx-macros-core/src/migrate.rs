#[cfg(any(sqlx_macros_unstable, procmacro2_semver_exempt))]
extern crate proc_macro;

use std::path::{Path, PathBuf};

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use sqlx_core::config::Config;
use sqlx_core::migrate::{Migration, MigrationType};
use syn::{Ident, LitStr};

pub const DEFAULT_PATH: &str = "./migrations";

pub struct QuoteMigrationType {
    crate_name: Ident,
    inner: MigrationType,
}

impl ToTokens for QuoteMigrationType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let crate_name = &self.crate_name;
        let ts = match self.inner {
            MigrationType::Simple => quote! { ::#crate_name::migrate::MigrationType::Simple },
            MigrationType::ReversibleUp => {
                quote! { ::#crate_name::migrate::MigrationType::ReversibleUp }
            }
            MigrationType::ReversibleDown => {
                quote! { ::#crate_name::migrate::MigrationType::ReversibleDown }
            }
        };
        tokens.append_all(ts);
    }
}

struct QuoteMigration {
    migration: Migration,
    path: PathBuf,
    crate_name: Ident,
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

        let crate_name = &self.crate_name;

        let migration_type = QuoteMigrationType {
            inner: *migration_type,
            crate_name: crate_name.clone(),
        };

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
            ::#crate_name::migrate::Migration {
                version: #version,
                description: ::std::borrow::Cow::Borrowed(#description),
                migration_type:  #migration_type,
                sql: ::#crate_name::SqlStr::from_static(#sql),
                no_tx: #no_tx,
                checksum: ::std::borrow::Cow::Borrowed(&[
                    #(#checksum),*
                ]),
            }
        };

        tokens.append_all(ts);
    }
}

pub fn default_path(config: &Config) -> &str {
    config
        .migrate
        .migrations_dir
        .as_deref()
        .unwrap_or(DEFAULT_PATH)
}

pub fn expand(path_arg: Option<LitStr>, crate_name: &Ident) -> crate::Result<TokenStream> {
    let config = Config::try_from_crate_or_default()?;

    let path = match path_arg {
        Some(path_arg) => crate::common::resolve_path(path_arg.value(), path_arg.span())?,
        None => { crate::common::resolve_path(default_path(&config), Span::call_site()) }?,
    };

    expand_with_path(&config, &path, crate_name)
}

pub fn expand_with_path(
    config: &Config,
    path: &Path,
    crate_name: &Ident,
) -> crate::Result<TokenStream> {
    let path = path.canonicalize().map_err(|e| {
        format!(
            "error canonicalizing migration directory {}: {e}",
            path.display()
        )
    })?;

    let resolve_config = config.migrate.to_resolve_config();

    // Use the same code path to resolve migrations at compile time and runtime.
    let migrations = sqlx_core::migrate::resolve_blocking_with_config(&path, &resolve_config)?
        .into_iter()
        .map(|(migration, path)| QuoteMigration {
            migration,
            path,
            crate_name: crate_name.clone(),
        });

    let table_name = config.migrate.table_name();

    let create_schemas = config.migrate.create_schemas.iter().map(|schema_name| {
        quote! { ::std::borrow::Cow::Borrowed(#schema_name) }
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
        ::#crate_name::migrate::Migrator {
            migrations: ::std::borrow::Cow::Borrowed(const {&[
                    #(#migrations),*
            ]}),
            create_schemas: ::std::borrow::Cow::Borrowed(&[#(#create_schemas),*]),
            table_name: ::std::borrow::Cow::Borrowed(#table_name),
            ..::#crate_name::migrate::Migrator::DEFAULT
        }
    })
}
