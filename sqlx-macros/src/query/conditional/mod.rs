use std::{fmt::Write, rc::Rc};
use std::any::Any;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{Error, Expr, Ident, parse::{Parse, ParseStream}, Pat, Path, Result, Token};
use syn::punctuated::Punctuated;

use segment::*;

mod map;
mod segment;


pub fn query_as(input: TokenStream) -> Result<TokenStream> {
    let input = syn::parse2::<Input>(input)?;
    let ctx = input.to_context()?;
    let out = ctx.generate_output();
    Ok(out)
}

/// Input to the conditional query macro.
struct Input {
    query_as: Path,
    segments: Vec<QuerySegment>,
    // separately specified arguments
    arguments: Vec<Expr>,
}

impl Input {
    /// Convert the input into a context
    fn to_context(&self) -> Result<Context> {
        let mut ctx = Context::Default(NormalContext {
            query_as: Rc::new(self.query_as.clone()),
            sql: String::new(),
            args: vec![],
        });

        for segment in &self.segments {
            ctx.add_segment(segment);
        }

        if ctx.branches() > 1 && !self.arguments.is_empty() {
            let err = Error::new(
                Span::call_site(),
                "branches (`match` and `if`) can only be used with inline arguments"
            );
            Err(err)
        } else {
            match &mut ctx {
                Context::Default(ctx) => ctx.args.extend(self.arguments.iter().cloned()),
                _ => unreachable!()
            }
            Ok(ctx)
        }
    }
}

impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self> {
        let query_as = input.parse::<Path>()?;
        input.parse::<Token![,]>()?;
        let segments = QuerySegment::parse_all(input)?;
        let arguments = match input.parse::<Option<Token![,]>>()? {
            None => vec![],
            Some(..) => Punctuated::<Expr, Token![,]>::parse_terminated(input)?.into_iter().collect()
        };
        Ok(Self {
            query_as,
            segments,
            arguments,
        })
    }
}

/// A context describes the current position within a conditional query.
#[derive(Clone)]
enum Context {
    Default(NormalContext),
    If(IfContext),
    Match(MatchContext),
}

trait IsContext {
    /// Return the number of branches the current context, including its children, contains.
    fn branches(&self) -> usize;
    /// Generate a call to a sqlx query macro for this context.
    fn to_query(&self, branches: usize, branch_counter: &mut usize) -> TokenStream;
    /// Add a piece of an SQL query to this context.
    fn add_sql(&mut self, sql: &SqlSegment);
}

impl IsContext for Context {
    fn branches(&self) -> usize {
        self.as_dyn().branches()
    }

    fn to_query(&self, branches: usize, branch_counter: &mut usize) -> TokenStream {
        self.as_dyn().to_query(branches, branch_counter)
    }

    fn add_sql(&mut self, sql: &SqlSegment) {
        self.as_dyn_mut().add_sql(sql);
    }
}

impl Context {
    fn generate_output(&self) -> TokenStream {
        let branches = self.branches();

        let result = {
            let mut branch_counter = 1;
            let output = self.to_query(branches, &mut branch_counter);
            assert_eq!(branch_counter, branches + 1);
            output
        };

        match branches {
            1 => quote!( #result ),
            _ => {
                let map = map::generate_conditional_map(branches);
                quote!( { #map #result } )
            }
        }
    }

    fn as_dyn(&self) -> &dyn IsContext {
        match self {
            Context::Default(c) => c as _,
            Context::If(c) => c as _,
            Context::Match(c) => c as _,
        }
    }

    fn as_dyn_mut(&mut self) -> &mut dyn IsContext {
        match self {
            Context::Default(c) => c as _,
            Context::If(c) => c as _,
            Context::Match(c) => c as _,
        }
    }

    fn add_segment(&mut self, s: &QuerySegment) {
        match s {
            QuerySegment::Sql(s) => self.add_sql(s),
            QuerySegment::If(s) => self.add_if(s),
            QuerySegment::Match(s) => self.add_match(s),
        }
    }

    fn add_if(&mut self, s: &IfSegment) {
        let mut if_ctx = IfContext {
            condition: s.condition.clone(),
            then: Box::new(self.clone()),
            or_else: Box::new(self.clone()),
        };
        for then in &s.then {
            if_ctx.then.add_segment(then);
        }
        for or_else in &s.or_else {
            if_ctx.or_else.add_segment(or_else);
        }
        // replace the current context with the new IfContext
        *self = Context::If(if_ctx);
    }

    fn add_match(&mut self, s: &MatchSegment) {
        let arms = s
            .arms
            .iter()
            .map(|arm| {
                let mut arm_ctx = MatchArmContext {
                    pattern: arm.pat.clone(),
                    inner: Box::new(self.clone()),
                };
                for segment in &arm.body {
                    arm_ctx.inner.add_segment(segment);
                }
                arm_ctx
            })
            .collect::<Vec<_>>();

        let match_ctx = MatchContext {
            expr: s.expr.clone(),
            arms,
        };

        // replace the current context with the new MatchContext
        *self = Context::Match(match_ctx);
    }
}

/// A "normal" linear context without any branches.
#[derive(Clone)]
struct NormalContext {
    query_as: Rc<Path>,
    sql: String,
    args: Vec<Expr>,
}

impl NormalContext {
    fn add_parameter(&mut self) {
        if cfg!(feature = "postgres") {
            write!(&mut self.sql, "${}", self.args.len() + 1).unwrap();
        } else {
            self.sql.push('?');
        }
    }
}

impl IsContext for NormalContext {
    fn branches(&self) -> usize {
        1
    }

    fn to_query(&self, branches: usize, branch_counter: &mut usize) -> TokenStream {
        let NormalContext {
            query_as,
            sql,
            args,
        } = self;
        *branch_counter += 1;

        let query_call = quote!(
            sqlx_macros::expand_query!(
                record = #query_as,
                source = #sql,
                args = [#(#args),*]
            )
        );
        match branches {
            1 => query_call,
            _ => {
                let variant = format_ident!("_{}", branch_counter);
                quote!(ConditionalMap::#variant(#query_call))
            }
        }
    }

    fn add_sql(&mut self, sql: &SqlSegment) {
        if !self.sql.is_empty() {
            self.sql.push(' ');
        }

        // push the new sql segment, replacing inline arguments (`"{some rust expression}"`)
        // with the appropriate placeholder (`$n` or `?`)
        let mut args = sql.args.iter();
        let mut arg = args.next();
        for (idx, c) in sql.sql.chars().enumerate() {
            if let Some((start, expr, end)) = arg {
                if idx < *start {
                    self.sql.push(c);
                }
                if idx == *end {
                    self.args.push(expr.clone());
                    self.add_parameter();
                    arg = args.next();
                }
            } else {
                self.sql.push(c);
            }
        }
    }
}

/// Context within an `if .. {..} else ..` clause.
#[derive(Clone)]
struct IfContext {
    condition: Expr,
    then: Box<Context>,
    or_else: Box<Context>,
}

impl IsContext for IfContext {
    fn branches(&self) -> usize {
        self.then.branches() + self.or_else.branches()
    }

    fn to_query(&self, branches: usize, branch_counter: &mut usize) -> TokenStream {
        let condition = &self.condition;
        let then = self.then.to_query(branches, branch_counter);
        let or_else = self.or_else.to_query(branches, branch_counter);
        quote! {
            if #condition {
                #then
            } else {
                #or_else
            }
        }
    }

    fn add_sql(&mut self, sql: &SqlSegment) {
        self.then.add_sql(sql);
        self.or_else.add_sql(sql);
    }
}

/// Context within `match .. { .. }`
#[derive(Clone)]
struct MatchContext {
    expr: Expr,
    arms: Vec<MatchArmContext>,
}

impl IsContext for MatchContext {
    fn branches(&self) -> usize {
        self.arms.iter().map(|arm| arm.branches()).sum()
    }

    fn to_query(&self, branches: usize, branch_counter: &mut usize) -> TokenStream {
        let expr = &self.expr;
        let arms = self.arms.iter().map(|arm| arm.to_query(branches, branch_counter));
        quote! {
            match #expr { #(#arms,)* }
        }
    }

    fn add_sql(&mut self, sql: &SqlSegment) {
        for arm in &mut self.arms {
            arm.add_sql(sql);
        }
    }
}

/// Context within the arm (`Pat => ..`) of a `match`
#[derive(Clone)]
struct MatchArmContext {
    pattern: Pat,
    inner: Box<Context>,
}

impl IsContext for MatchArmContext {
    fn branches(&self) -> usize {
        self.inner.branches()
    }

    fn to_query(&self, branches: usize, branch_counter: &mut usize) -> TokenStream {
        let pat = &self.pattern;
        let inner = self.inner.to_query(branches, branch_counter);
        quote! {
            #pat => #inner
        }
    }

    fn add_sql(&mut self, sql: &SqlSegment) {
        self.inner.add_sql(sql);
    }
}

#[cfg(test)]
mod tests {
    use crate::query::conditional::Input;
    use quote::quote;

    #[test]
    fn simple() {
        let input = quote! {
            OptionalRecord, "select owner_id as `id: _` from tweet"
        };
        let result = syn::parse2::<Input>(input).unwrap();
        println!("{}", result.segments.len());
    }
}
