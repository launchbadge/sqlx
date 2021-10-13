use syn::{
    braced,
    Error,
    Expr, LitStr, parse::{Parse, ParseStream, Peek}, Pat, Token,
};

/// A single "piece" of the input.
pub enum QuerySegment {
    /// A part of an SQL query, like `"SELECT *"`
    Sql(SqlSegment),
    /// An `if .. { .. }`, with optional `else ..`
    If(IfSegment),
    /// An exhaustive `match .. { .. }`
    Match(MatchSegment),
    /// A query argument. Can be an arbitrary expression, prefixed by `?`, like `?search.trim()`
    Arg(ArgSegment),
}

impl QuerySegment {
    /// Parse segments up to the first occurrence of the given token, or until the input is empty.
    fn parse_until<T: Peek>(input: ParseStream, until: T) -> syn::Result<Vec<Self>> {
        let mut segments = vec![];
        while !input.is_empty() && !input.peek(until) {
            segments.push(QuerySegment::parse(input)?)
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

pub struct ArgSegment {
    pub argument: Expr,
}

impl ArgSegment {
    const EXPECT: &'static str = "?..";

    fn matches(input: ParseStream) -> bool {
        input.peek(Token![?])
    }
}

impl Parse for ArgSegment {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![?]>()?;
        Ok(Self {
            argument: input.parse::<Expr>()?,
        })
    }
}

pub struct SqlSegment {
    pub query: String,
}

impl SqlSegment {
    const EXPECT: &'static str = "\"..\"";

    fn matches(input: &ParseStream) -> bool {
        input.fork().parse::<LitStr>().is_ok()
    }
}

impl Parse for SqlSegment {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lit = input.parse::<LitStr>()?;
        Ok(Self { query: lit.value() })
    }
}

pub struct MatchSegment {
    pub expr: Expr,
    pub arms: Vec<MatchSegmentArm>,
}

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
        if SqlSegment::matches(&input) {
            Ok(QuerySegment::Sql(SqlSegment::parse(input)?))
        } else if IfSegment::matches(&input) {
            Ok(QuerySegment::If(IfSegment::parse(input)?))
        } else if MatchSegment::matches(input) {
            Ok(QuerySegment::Match(MatchSegment::parse(input)?))
        } else if ArgSegment::matches(input) {
            Ok(QuerySegment::Arg(ArgSegment::parse(input)?))
        } else {
            let error = format!(
                "expected `{}`, `{}`, `{}` or `{}`",
                SqlSegment::EXPECT,
                IfSegment::EXPECT,
                MatchSegment::EXPECT,
                ArgSegment::EXPECT
            );
            Err(Error::new(input.span(), error))
        }
    }
}
