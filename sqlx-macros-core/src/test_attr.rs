use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::LitStr;

struct Args {
    fixtures: Vec<LitStr>,
    migrations: MigrationsOpt,
}

enum MigrationsOpt {
    InferredPath,
    ExplicitPath(LitStr),
    ExplicitMigrator(syn::Path),
    Disabled,
}

pub fn expand(args: syn::AttributeArgs, input: syn::ItemFn) -> crate::Result<TokenStream> {
    if input.sig.inputs.is_empty() {
        if !args.is_empty() {
            if cfg!(feature = "migrate") {
                return Err(syn::Error::new_spanned(
                    args.first().unwrap(),
                    "control attributes are not allowed unless \
                        the `migrate` feature is enabled and \
                        automatic test DB management is used; see docs",
                )
                .into());
            }

            return Err(syn::Error::new_spanned(
                args.first().unwrap(),
                "control attributes are not allowed unless \
                    automatic test DB management is used; see docs",
            )
            .into());
        }

        return Ok(expand_simple(input));
    }

    #[cfg(feature = "migrate")]
    return expand_advanced(args, input);

    #[cfg(not(feature = "migrate"))]
    return Err(syn::Error::new_spanned(input, "`migrate` feature required").into());
}

fn expand_simple(input: syn::ItemFn) -> TokenStream {
    let ret = &input.sig.output;
    let name = &input.sig.ident;
    let body = &input.block;
    let attrs = &input.attrs;

    quote! {
        #[::core::prelude::v1::test]
        #(#attrs)*
        fn #name() #ret {
            ::sqlx::test_block_on(async { #body })
        }
    }
}

#[cfg(feature = "migrate")]
fn expand_advanced(args: syn::AttributeArgs, input: syn::ItemFn) -> crate::Result<TokenStream> {
    let ret = &input.sig.output;
    let name = &input.sig.ident;
    let inputs = &input.sig.inputs;
    let body = &input.block;
    let attrs = &input.attrs;

    let args = parse_args(args)?;

    let fn_arg_types = inputs.iter().map(|_| quote! { _ });

    let fixtures = args.fixtures.into_iter().map(|fixture| {
        let path = format!("fixtures/{}.sql", fixture.value());

        quote! {
            ::sqlx::testing::TestFixture {
                path: #path,
                contents: include_str!(#path),
            }
        }
    });

    let migrations = match args.migrations {
        MigrationsOpt::ExplicitPath(path) => {
            let migrator = crate::migrate::expand_migrator_from_lit_dir(path)?;
            quote! { args.migrator(&#migrator); }
        }
        MigrationsOpt::InferredPath if !inputs.is_empty() => {
            let migrations_path = crate::common::resolve_path("./migrations", Span::call_site())?;

            if migrations_path.is_dir() {
                let migrator = crate::migrate::expand_migrator(&migrations_path)?;
                quote! { args.migrator(&#migrator); }
            } else {
                quote! {}
            }
        }
        MigrationsOpt::ExplicitMigrator(path) => {
            quote! { args.migrator(&#path); }
        }
        _ => quote! {},
    };

    Ok(quote! {
        #[::core::prelude::v1::test]
        #(#attrs)*
        fn #name() #ret {
            async fn inner(#inputs) #ret {
                #body
            }

            let mut args = ::sqlx::testing::TestArgs::new(concat!(module_path!(), "::", stringify!(#name)));

            #migrations

            args.fixtures(&[#(#fixtures),*]);

            // We need to give a coercion site or else we get "unimplemented trait" errors.
            let f: fn(#(#fn_arg_types),*) -> _ = inner;

            ::sqlx::testing::TestFn::run_test(f, args)
        }
    })
}

#[cfg(feature = "migrate")]
fn parse_args(attr_args: syn::AttributeArgs) -> syn::Result<Args> {
    let mut fixtures = vec![];
    let mut migrations = MigrationsOpt::InferredPath;

    for arg in attr_args {
        match arg {
            syn::NestedMeta::Meta(syn::Meta::List(list)) if list.path.is_ident("fixtures") => {
                if !fixtures.is_empty() {
                    return Err(syn::Error::new_spanned(list, "duplicate `fixtures` arg"));
                }

                for nested in list.nested {
                    match nested {
                        syn::NestedMeta::Lit(syn::Lit::Str(litstr)) => fixtures.push(litstr),
                        other => {
                            return Err(syn::Error::new_spanned(other, "expected string literal"))
                        }
                    }
                }
            }
            syn::NestedMeta::Meta(syn::Meta::NameValue(namevalue))
                if namevalue.path.is_ident("migrations") =>
            {
                if !matches!(migrations, MigrationsOpt::InferredPath) {
                    return Err(syn::Error::new_spanned(
                        namevalue,
                        "cannot have more than one `migrations` or `migrator` arg",
                    ));
                }

                migrations = match namevalue.lit {
                    syn::Lit::Bool(litbool) => {
                        if !litbool.value {
                            // migrations = false
                            MigrationsOpt::Disabled
                        } else {
                            // migrations = true
                            return Err(syn::Error::new_spanned(
                                litbool,
                                "`migrations = true` is redundant",
                            ));
                        }
                    }
                    // migrations = "<path>"
                    syn::Lit::Str(litstr) => MigrationsOpt::ExplicitPath(litstr),
                    _ => {
                        return Err(syn::Error::new_spanned(
                            namevalue,
                            "expected string or `false`",
                        ))
                    }
                };
            }
            syn::NestedMeta::Meta(syn::Meta::NameValue(namevalue))
                if namevalue.path.is_ident("migrator") =>
                {
                    if !matches!(migrations, MigrationsOpt::InferredPath) {
                        return Err(syn::Error::new_spanned(
                            namevalue,
                            "cannot have more than one `migrations` or `migrator` arg",
                        ));
                    }

                    migrations = match namevalue.lit {
                        // migrator = "<path>"
                        syn::Lit::Str(litstr) => MigrationsOpt::ExplicitMigrator(litstr.parse()?),
                        _ => {
                            return Err(syn::Error::new_spanned(
                                namevalue,
                                "expected string",
                            ))
                        }
                    };
                }
            other => {
                return Err(syn::Error::new_spanned(
                    other,
                    "expected `fixtures(\"<filename>\", ...)` or `migrations = \"<path>\" | false` or `migrator = \"<rust path>\"`",
                ))
            }
        }
    }

    Ok(Args {
        fixtures,
        migrations,
    })
}
