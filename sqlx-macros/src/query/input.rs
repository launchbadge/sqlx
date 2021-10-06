use std::fs;

use once_cell::sync::Lazy;
use proc_macro2::{Ident, Span};
use regex::Regex;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Expr, LitBool, LitStr, Token};
use syn::{ExprArray, Type};

/// Macro input shared by `query!()` and `query_file!()`
pub struct QueryMacroInput {
    pub(super) sql: String,

    #[cfg_attr(not(feature = "offline"), allow(dead_code))]
    pub(super) src_span: Span,

    pub(super) record_type: RecordType,

    pub(super) arg_exprs: Vec<Expr>,

    pub(super) checked: bool,

    pub(super) file_path: Option<String>,
}

enum QuerySrc {
    String(String),
    File(String),
}

pub enum RecordType {
    Given(Type),
    Scalar,
    Generated,
}

impl Parse for QueryMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut query_src: Option<(QuerySrc, Span)> = None;
        let mut args: Option<Vec<Expr>> = None;
        let mut record_type = RecordType::Generated;
        let mut checked = true;
        let mut query_name: Option<(String, Span)> = None;

        let mut expect_comma = false;

        while !input.is_empty() {
            if expect_comma {
                let _ = input.parse::<syn::token::Comma>()?;
            }

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
                if !matches!(record_type, RecordType::Generated) {
                    return Err(input.error("colliding `scalar` or `record` key"));
                }

                record_type = RecordType::Given(input.parse()?);
            } else if key == "scalar" {
                if !matches!(record_type, RecordType::Generated) {
                    return Err(input.error("colliding `scalar` or `record` key"));
                }

                // we currently expect only `scalar = _`
                // a `query_as_scalar!()` variant seems less useful than just overriding the type
                // of the column in SQL
                input.parse::<syn::Token![_]>()?;
                record_type = RecordType::Scalar;
            } else if key == "checked" {
                let lit_bool = input.parse::<LitBool>()?;
                checked = lit_bool.value;
            } else if key == "query_name" {
                let ident = input.parse::<Ident>()?;
                query_name = Some((ident.to_string(), ident.span()));
            } else {
                let message = format!("unexpected input key: {}", key);
                return Err(syn::Error::new_spanned(key, message));
            }

            expect_comma = true;
        }

        let (src, src_span) =
            query_src.ok_or_else(|| input.error("expected `source` or `source_file` key"))?;

        if !matches!(&src, QuerySrc::File(_)) && query_name.is_some() {
            return Err(input.error("`query_name` must be used with `source_file`"));
        }

        let arg_exprs = args.unwrap_or_default();

        let file_path = src.file_path(src_span)?;

        Ok(QueryMacroInput {
            sql: src.resolve(
                src_span,
                query_name
                    .as_ref()
                    .map(|(name, span)| (name.as_str(), *span)),
            )?,
            src_span,
            record_type,
            arg_exprs,
            checked,
            file_path,
        })
    }
}

impl QuerySrc {
    /// If the query source is a file, read it to a string. Otherwise return the query string.
    fn resolve(self, source_span: Span, query_name: Option<(&str, Span)>) -> syn::Result<String> {
        match self {
            QuerySrc::String(string) => Ok(string),
            QuerySrc::File(file) => read_file_src(&file, source_span, query_name),
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

fn extract_query_by_name(
    source_span: Span,
    sql_content: &str,
    query_name: &str,
    query_id_span: Span,
) -> syn::Result<String> {
    static HEADER_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"(?m)^[\r\t ]*---*[\r\t ]*name[\r\t ]*[:=][\r\t ]*(\w+)\b.*$"#).unwrap()
    });
    static HEADER_START_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"^[\r\t ]*---*[\r\t ]*name[\r\t ]*[:=]"#).unwrap());

    if !query_name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(syn::Error::new(
            query_id_span,
            "the query name is invalid (allowed characters are [A-Za-z0-9_]",
        ));
    }

    let query_header_idx = HEADER_RE
        .captures_iter(sql_content)
        .find_map(|captures| {
            let file_query_name = captures.get(1).unwrap();
            if file_query_name.as_str() == query_name {
                Some(file_query_name.start())
            } else {
                None
            }
        })
        .ok_or_else(|| {
            syn::Error::new(
                source_span,
                format!(
                    "the query file does not contain a query with name `{}`",
                    query_name
                ),
            )
        })?;

    let mut lines = (&sql_content[query_header_idx..]).split_inclusive('\n');

    let query_idx = lines
        .next()
        .ok_or_else(|| {
            syn::Error::new(
                source_span,
                format!("the query with name `{}` is empty", query_name),
            )
        })?
        .len()
        + query_header_idx;
    let query_len = lines
        .take_while(|line| !HEADER_START_RE.is_match(line))
        .map(|line| line.len())
        .sum::<usize>();

    if query_len == 0 {
        return Err(syn::Error::new(
            source_span,
            format!("the query with name `{}` is empty", query_name),
        ));
    }

    let query = &sql_content[query_idx..(query_idx + query_len)];
    if query.is_empty() || query.trim().is_empty() {
        return Err(syn::Error::new(
            source_span,
            format!("the query with name `{}` is empty", query_name),
        ));
    }

    Ok(query.to_string())
}

fn read_file_src(
    source: &str,
    source_span: Span,
    query_name: Option<(&str, Span)>,
) -> syn::Result<String> {
    let file_path = crate::common::resolve_path(source, source_span)?;

    let content = fs::read_to_string(&file_path).map_err(|e| {
        syn::Error::new(
            source_span,
            format!(
                "failed to read query file at {}: {}",
                file_path.display(),
                e
            ),
        )
    })?;

    if let Some((query_name, query_id_span)) = query_name {
        return extract_query_by_name(source_span, &content, query_name, query_id_span);
    }

    Ok(content)
}
