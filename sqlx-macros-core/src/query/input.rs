use std::fs;

use proc_macro2::{Ident, Span};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{bracketed, Expr, LitBool, LitStr, Meta, Token};
use syn::{ExprArray, Type};

/// Macro input shared by `query!()` and `query_file!()`
pub struct QueryMacroInput {
    pub(super) sql: String,

    pub(super) src_span: Span,

    pub(super) output_type: OutputType,

    pub(super) arg_exprs: Vec<Expr>,

    pub(super) checked: bool,

    pub(super) file_path: Option<String>,
}

enum QuerySrc {
    String(String),
    File(String),
}

pub enum OutputType {
    GivenRecord(Type),
    Scalar,
    GeneratedRecord(Vec<Meta>),
}

impl Parse for QueryMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut query_src: Option<(QuerySrc, Span)> = None;
        let mut args: Option<Vec<Expr>> = None;
        let mut output_type = OutputType::GeneratedRecord(Vec::new());
        let mut checked = true;

        while !input.is_empty() {
            let key: Ident = input.parse()?;

            let _ = input.parse::<syn::token::Eq>()?;

            if key == "source" {
                let span = input.span();
                let query_str = Punctuated::<LitStr, Token![+]>::parse_separated_nonempty(input)?
                    .iter()
                    .map(LitStr::value)
                    .collect();
                query_src = Some((QuerySrc::String(query_str), span));
            } else if key == "source_file" {
                let lit_str = input.parse::<LitStr>()?;
                query_src = Some((QuerySrc::File(lit_str.value()), lit_str.span()));
            } else if key == "args" {
                let exprs = input.parse::<ExprArray>()?;
                args = Some(exprs.elems.into_iter().collect())
            } else if key == "record" {
                if !matches!(output_type, OutputType::GeneratedRecord(_)) {
                    return Err(input.error("colliding `scalar` or `record` key"));
                }

                output_type = OutputType::GivenRecord(input.parse()?);
            } else if key == "scalar" {
                if !matches!(output_type, OutputType::GeneratedRecord(_)) {
                    return Err(input.error("colliding `scalar` or `record` key"));
                }

                // we currently expect only `scalar = _`
                // a `query_as_scalar!()` variant seems less useful than just overriding the type
                // of the column in SQL
                input.parse::<syn::Token![_]>()?;
                output_type = OutputType::Scalar;
            } else if key == "attrs" {
                let OutputType::GeneratedRecord(ref mut attrs) = output_type else {
                    return Err(input.error("can only set attributes for generated type"));
                };
                let content;
                bracketed!(content in input);
                *attrs = Punctuated::<Meta, Token![,]>::parse_terminated(&content)?
                    .into_iter()
                    .collect();
            } else if key == "checked" {
                let lit_bool = input.parse::<LitBool>()?;
                checked = lit_bool.value;
            } else {
                let message = format!("unexpected input key: {key}");
                return Err(syn::Error::new_spanned(key, message));
            }

            if input.is_empty() {
                break;
            } else {
                input.parse::<Token![,]>()?;
            }
        }

        let (src, src_span) =
            query_src.ok_or_else(|| input.error("expected `source` or `source_file` key"))?;

        let arg_exprs = args.unwrap_or_default();

        let file_path = src.file_path(src_span)?;

        Ok(QueryMacroInput {
            sql: src.resolve(src_span)?,
            src_span,
            output_type,
            arg_exprs,
            checked,
            file_path,
        })
    }
}

impl QuerySrc {
    /// If the query source is a file, read it to a string. Otherwise return the query string.
    fn resolve(self, source_span: Span) -> syn::Result<String> {
        match self {
            QuerySrc::String(string) => Ok(string),
            QuerySrc::File(file) => read_file_src(&file, source_span),
        }
    }

    fn file_path(&self, source_span: Span) -> syn::Result<Option<String>> {
        if let QuerySrc::File(ref file) = *self {
            let path = crate::common::resolve_path(file, source_span)?
                .canonicalize()
                .map_err(|e| syn::Error::new(source_span, e))?;

            Ok(Some(
                path.to_str()
                    .ok_or_else(|| {
                        syn::Error::new(
                            source_span,
                            "query file path cannot be represented as a string",
                        )
                    })?
                    .to_string(),
            ))
        } else {
            Ok(None)
        }
    }
}

fn read_file_src(source: &str, source_span: Span) -> syn::Result<String> {
    let file_path = crate::common::resolve_path(source, source_span)?;

    fs::read_to_string(&file_path).map_err(|e| {
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
