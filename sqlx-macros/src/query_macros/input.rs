use std::env;

use proc_macro2::{Ident, Span, TokenStream};
use sqlx::runtime::fs;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Group;
use syn::{Expr, ExprLit, ExprPath, Lit};
use syn::{ExprGroup, Token};

use quote::{format_ident, quote, ToTokens};

use sqlx::describe::Describe;
use sqlx::Connection;

/// Macro input shared by `query!()` and `query_file!()`
pub struct QueryMacroInput {
    pub(super) source: String,
    pub(super) source_span: Span,
    // `arg0 .. argN` for N arguments
    pub(super) arg_names: Vec<Ident>,
    pub(super) arg_exprs: Vec<Expr>,
}

impl QueryMacroInput {
    fn from_exprs(input: ParseStream, mut args: impl Iterator<Item = Expr>) -> syn::Result<Self> {
        fn lit_err<T>(span: Span, unexpected: Expr) -> syn::Result<T> {
            Err(syn::Error::new(
                span,
                format!(
                    "expected string literal, got {}",
                    unexpected.to_token_stream()
                ),
            ))
        }

        let (source, source_span) = match args.next() {
            Some(Expr::Lit(ExprLit {
                lit: Lit::Str(sql), ..
            })) => (sql.value(), sql.span()),
            Some(Expr::Group(ExprGroup {
                expr,
                group_token: Group { span },
                ..
            })) => {
                // this duplication with the above is necessary because `expr` is `Box<Expr>` here
                // which we can't directly pattern-match without `box_patterns`
                match *expr {
                    Expr::Lit(ExprLit {
                        lit: Lit::Str(sql), ..
                    }) => (sql.value(), span),
                    other_expr => return lit_err(span, other_expr),
                }
            }
            Some(other_expr) => return lit_err(other_expr.span(), other_expr),
            None => return Err(input.error("expected SQL string literal")),
        };

        let arg_exprs: Vec<_> = args.collect();
        let arg_names = (0..arg_exprs.len())
            .map(|i| format_ident!("arg{}", i))
            .collect();

        Ok(Self {
            source,
            source_span,
            arg_exprs,
            arg_names,
        })
    }

    pub async fn expand_file_src(self) -> syn::Result<Self> {
        let source = read_file_src(&self.source, self.source_span).await?;

        Ok(Self { source, ..self })
    }

    /// Run a parse/describe on the query described by this input and validate that it matches the
    /// passed number of args
    pub async fn describe_validate<C: Connection>(
        &self,
        conn: &mut C,
    ) -> crate::Result<Describe<C::Database>> {
        let describe = conn
            .describe(&self.source)
            .await
            .map_err(|e| syn::Error::new(self.source_span, e))?;

        if self.arg_names.len() != describe.param_types.len() {
            return Err(syn::Error::new(
                Span::call_site(),
                format!(
                    "expected {} parameters, got {}",
                    describe.param_types.len(),
                    self.arg_names.len()
                ),
            )
            .into());
        }

        Ok(describe)
    }
}

impl Parse for QueryMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args = Punctuated::<Expr, Token![,]>::parse_terminated(input)?.into_iter();

        Self::from_exprs(input, args)
    }
}

/// Macro input shared by `query_as!()` and `query_file_as!()`
pub struct QueryAsMacroInput {
    pub(super) as_ty: ExprPath,
    pub(super) query_input: QueryMacroInput,
}

impl QueryAsMacroInput {
    pub async fn expand_file_src(self) -> syn::Result<Self> {
        Ok(Self {
            query_input: self.query_input.expand_file_src().await?,
            ..self
        })
    }
}

impl Parse for QueryAsMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        fn path_err<T>(span: Span, unexpected: Expr) -> syn::Result<T> {
            Err(syn::Error::new(
                span,
                format!(
                    "expected path to a type, got {}",
                    unexpected.to_token_stream()
                ),
            ))
        }

        let mut args = Punctuated::<Expr, Token![,]>::parse_terminated(input)?.into_iter();

        let as_ty = match args.next() {
            Some(Expr::Path(path)) => path,
            Some(Expr::Group(ExprGroup {
                expr,
                group_token: Group { span },
                ..
            })) => {
                // this duplication with the above is necessary because `expr` is `Box<Expr>` here
                // which we can't directly pattern-match without `box_patterns`
                match *expr {
                    Expr::Path(path) => path,
                    other_expr => return path_err(span, other_expr),
                }
            }
            Some(other_expr) => return path_err(other_expr.span(), other_expr),
            None => return Err(input.error("expected path to SQL file")),
        };

        Ok(QueryAsMacroInput {
            as_ty,
            query_input: QueryMacroInput::from_exprs(input, args)?,
        })
    }
}

async fn read_file_src(source: &str, source_span: Span) -> syn::Result<String> {
    use std::path::Path;

    let path = Path::new(source);

    if path.is_absolute() {
        return Err(syn::Error::new(
            source_span,
            "absolute paths will only work on the current machine",
        ));
    }

    // requires `proc_macro::SourceFile::path()` to be stable
    // https://github.com/rust-lang/rust/issues/54725
    if path.is_relative()
        && !path
            .parent()
            .map_or(false, |parent| !parent.as_os_str().is_empty())
    {
        return Err(syn::Error::new(
            source_span,
            "paths relative to the current file's directory are not currently supported",
        ));
    }

    let base_dir = env::var("CARGO_MANIFEST_DIR").map_err(|_| {
        syn::Error::new(
            source_span,
            "CARGO_MANIFEST_DIR is not set; please use Cargo to build",
        )
    })?;

    let base_dir_path = Path::new(&base_dir);

    let file_path = base_dir_path.join(path);

    fs::read_to_string(&file_path).await.map_err(|e| {
        syn::Error::new(
            source_span,
            format!(
                "failed to read query file at {}: {}",
                file_path.display(),
                e
            ),
        )
    })
}
