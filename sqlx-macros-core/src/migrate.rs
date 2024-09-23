#[cfg(any(sqlx_macros_unstable, procmacro2_semver_exempt))]
extern crate proc_macro;

use std::path::{Path, PathBuf};

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::LitStr;
use syn::spanned::Spanned;
use sqlx_core::config::Config;
use sqlx_core::migrate::{Migration, MigrationType, ResolveConfig};

pub const DEFAULT_PATH: &str = "./migrations";

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

pub fn default_path(config: &Config) -> &str {
    config.migrate.migrations_dir
        .as_deref()
        .unwrap_or(DEFAULT_PATH)
}

pub fn expand(path_arg: Option<LitStr>) -> crate::Result<TokenStream> {
    let config = Config::from_crate();

   let path = match path_arg {
       Some(path_arg) => crate::common::resolve_path(path_arg.value(), path_arg.span())?,
       None => {
           crate::common::resolve_path(default_path(config), Span::call_site())
       }?
   };

    expand_with_path(config, &path)
}

pub fn expand_with_path(config: &Config, path: &Path) -> crate::Result<TokenStream> {
    let path = path.canonicalize().map_err(|e| {
        format!(
            "error canonicalizing migration directory {}: {e}",
            path.display()
        )
    })?;

    let mut resolve_config = ResolveConfig::new();
    resolve_config.ignore_chars(&config.migrate.ignored_chars);

    // Use the same code path to resolve migrations at compile time and runtime.
    let migrations = sqlx_core::migrate::resolve_blocking_with_config(&path, &resolve_config)?
        .into_iter()
        .map(|(migration, path)| QuoteMigration { migration, path });

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

    let table_name = config.migrate.table_name
        .as_deref()
        .map_or_else(
            || quote! {},
            |name| quote! { table_name: Some(::std::borrow::Cow::Borrowed(#name)), }
        );

    Ok(quote! {
        ::sqlx::migrate::Migrator {
            migrations: ::std::borrow::Cow::Borrowed(&[
                    #(#migrations),*
            ]),
            #table_name
            ..::sqlx::migrate::Migrator::DEFAULT
        }
    })
}
