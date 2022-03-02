use proc_macro2::Span;
use std::mem::swap;
use std::ptr::replace;
use syn::{
    braced,
    parse::{Parse, ParseStream, Peek},
    Error, Expr, LitStr, Pat, Token,
};

/// A single "piece" of the input.
#[derive(Debug)]
pub enum QuerySegment {
    /// A part of an SQL query, like `"SELECT *"`
    Sql(SqlSegment),
    /// An `if .. { .. }`, with optional `else ..`
    If(IfSegment),
    /// An exhaustive `match .. { .. }`
    Match(MatchSegment),
}

impl QuerySegment {
    /// Parse segments up to the first occurrence of the given token, or until the input is empty.
    pub fn parse_until<T: Peek>(input: ParseStream, until: T) -> syn::Result<Vec<Self>> {
        let mut segments = vec![];
        while !input.is_empty() && !input.peek(until) {
            segments.push(QuerySegment::parse(input)?);
        }
        Ok(segments)
    }

    /// Parse segments until the input is empty.
    pub fn parse_all(input: ParseStream) -> syn::Result<Vec<Self>> {
        let mut segments = vec![];
        while !input.is_empty() {
            segments.push(QuerySegment::parse(input)?);
        }
        Ok(segments)
    }
}

#[derive(Debug)]
pub struct SqlSegment {
    pub sql: String,
    pub args: Vec<(usize, Expr, usize)>,
}

impl SqlSegment {
    const EXPECT: &'static str = "\"..\"";

    fn matches(input: &ParseStream) -> bool {
        input.fork().parse::<LitStr>().is_ok()
    }
}

impl Parse for SqlSegment {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let sql = input.parse::<LitStr>()?.value();
        let args = parse_inline_args(&sql)?;

        Ok(Self { sql, args })
    }
}

// parses inline arguments in the query, for example `".. WHERE user_id = {1}"`, returning them with
// the index of `{`, the parsed argument, and the index of the `}`.
fn parse_inline_args(sql: &str) -> syn::Result<Vec<(usize, Expr, usize)>> {
    let mut args = vec![];
    let mut curr_level = 0;
    let mut curr_arg = None;

    for (idx, c) in sql.chars().enumerate() {
        match c {
            '{' => {
                if curr_arg.is_none() {
                    curr_arg = Some((idx, String::new()));
                }
                curr_level += 1;
            }
            '}' => {
                if curr_arg.is_none() {
                    let err = Error::new(Span::call_site(), "unexpected '}' in query string");
                    return Err(err);
                };
                if curr_level == 1 {
                    let (arg_start, arg_str) = std::mem::replace(&mut curr_arg, None).unwrap();
                    let arg = syn::parse_str(&arg_str)?;
                    args.push((arg_start, arg, idx));
                }
                curr_level -= 1;
            }
            c => {
                if let Some((_, arg)) = &mut curr_arg {
                    arg.push(c);
                }
            }
        }
    }

    if curr_arg.is_some() {
        let err = Error::new(Span::call_site(), "expected '}', but got end of string");
        return Err(err);
    }

    Ok(args)
}

#[derive(Debug)]
pub struct MatchSegment {
    pub expr: Expr,
    pub arms: Vec<MatchSegmentArm>,
}

#[derive(Debug)]
pub struct MatchSegmentArm {
    pub pat: Pat,
    pub body: Vec<QuerySegment>,
}

impl MatchSegmentArm {
    fn parse_all(input: ParseStream) -> syn::Result<Vec<Self>> {
        let mut arms = vec![];
        while !input.is_empty() {
            arms.push(Self::parse(input)?);
        }
        Ok(arms)
    }
}

impl Parse for MatchSegmentArm {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let pat = input.parse()?;
        input.parse::<Token![=>]>()?;
        let body = if input.peek(syn::token::Brace) {
            let body;
            braced!(body in input);
            QuerySegment::parse_all(&body)?
        } else {
            QuerySegment::parse_until(input, Token![,])?
        };
        input.parse::<Option<Token![,]>>()?;
        Ok(Self { pat, body })
    }
}

impl MatchSegment {
    const EXPECT: &'static str = "match .. { .. }";

    fn matches(input: ParseStream) -> bool {
        input.peek(Token![match])
    }
}

impl Parse for MatchSegment {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![match]>()?;
        let expr = input.call(Expr::parse_without_eager_brace)?;
        let input = {
            let content;
            braced!(content in input);
            content
        };
        let arms = MatchSegmentArm::parse_all(&input)?;

        Ok(Self { expr, arms })
    }
}

#[derive(Debug)]
pub struct IfSegment {
    pub condition: Expr,
    pub then: Vec<QuerySegment>,
    pub or_else: Vec<QuerySegment>,
}

impl IfSegment {
    const EXPECT: &'static str = "if { .. }";

    fn matches(input: ParseStream) -> bool {
        input.peek(Token![if])
    }
}

impl Parse for IfSegment {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![if]>()?;
        let condition = input.call(Expr::parse_without_eager_brace)?;
        let then = {
            let if_then;
            braced!(if_then in input);
            QuerySegment::parse_all(&if_then)?
        };
        let or_else = if input.parse::<Option<Token![else]>>()?.is_some() {
            if IfSegment::matches(input) {
                let else_if = IfSegment::parse(input)?;
                vec![QuerySegment::If(else_if)]
            } else {
                let or_else;
                braced!(or_else in input);
                QuerySegment::parse_all(&or_else)?
            }
        } else {
            vec![]
        };
        Ok(Self {
            condition,
            then,
            or_else,
        })
    }
}

impl Parse for QuerySegment {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // parse optional '+' for backwards compatibility
        if input.peek(Token![+]) {
            input.parse::<Token![+]>()?;
        }

        if SqlSegment::matches(&input) {
            Ok(QuerySegment::Sql(input.parse()?))
        } else if IfSegment::matches(&input) {
            Ok(QuerySegment::If(input.parse()?))
        } else if MatchSegment::matches(input) {
            Ok(QuerySegment::Match(input.parse()?))
        } else {
            let error = format!(
                "expected `{}`, `{}` or `{}`",
                SqlSegment::EXPECT,
                IfSegment::EXPECT,
                MatchSegment::EXPECT,
            );
            Err(Error::new(input.span(), error))
        }
    }
}
