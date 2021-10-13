use std::rc::Rc;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    Expr,
    Ident, parse::{Parse, ParseStream}, Pat, Path, Result, Token,
};

use segment::*;

mod map;
mod segment;

/// Expand a call to `conditional_query_as!`
pub fn conditional_query_as(input: TokenStream) -> Result<TokenStream> {
    let input = syn::parse2::<Input>(input)?;
    let ctx = input.to_context();
    let out = ctx.generate_output(input.testing);
    Ok(out)
}

/// Input to the conditional query macro.
struct Input {
    /// `true` if the macro should only output information about the query instead of actually
    /// calling a query macro
    testing: bool,
    query_as: Path,
    segments: Vec<QuerySegment>,
}

impl Input {
    /// Convert the input into a context
    fn to_context(&self) -> Context {
        let mut ctx = Context::Default(NormalContext {
            query_as: Rc::new(self.query_as.clone()),
            sql: String::new(),
            args: vec![],
        });

        for segment in &self.segments {
            ctx.add_segment(segment);
        }

        ctx
    }
}

syn::custom_keyword!(testing);

impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self> {
        let testing = input.parse::<Option<testing>>()?.is_some();
        let query_as = input.parse::<Path>()?;
        input.parse::<Token![,]>()?;
        Ok(Self {
            testing,
            query_as,
            segments: QuerySegment::parse_all(input)?,
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
    fn to_query(&self, testing: bool, branch_counter: &mut usize) -> TokenStream;
    /// Add a piece of an SQL query to this context.
    fn add_sql(&mut self, sql: &SqlSegment);
    /// Add an argument to this context.
    fn add_arg(&mut self, arg: &ArgSegment);
}

impl IsContext for Context {
    fn branches(&self) -> usize {
        self.as_dyn().branches()
    }

    fn to_query(&self, testing: bool, branch_counter: &mut usize) -> TokenStream {
        self.as_dyn().to_query(testing, branch_counter)
    }

    fn add_sql(&mut self, sql: &SqlSegment) {
        self.as_dyn_mut().add_sql(sql);
    }

    fn add_arg(&mut self, arg: &ArgSegment) {
        self.as_dyn_mut().add_arg(arg);
    }
}

impl Context {
    fn generate_output(&self, testing: bool) -> TokenStream {
        let branches = self.branches();

        let result = {
            let mut branch_counter = 1;
            let output = self.to_query(testing, &mut branch_counter);
            assert_eq!(branch_counter, branches + 1);
            output
        };

        if testing {
            quote!( #result )
        } else {
            let map = map::generate_conditional_map(branches);
            quote!( { #map #result } )
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
            QuerySegment::Arg(s) => self.add_arg(s),
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

impl IsContext for NormalContext {
    fn branches(&self) -> usize {
        1
    }

    fn to_query(&self, testing: bool, branch_counter: &mut usize) -> TokenStream {
        let NormalContext { query_as, sql, args } = self;
        *branch_counter += 1;

        if testing {
            quote! {
                (stringify!(#query_as), #sql, vec![#(stringify!(#args)),*])
                    as (&'static str, &'static str, Vec<&'static str>)
            }
        } else {
            let variant = Ident::new(&format!("_{}", branch_counter), Span::call_site());
            quote!(ConditionalMap::#variant(sqlx::query_as!(#query_as, #sql, #(#args),*)))
        }
    }

    fn add_sql(&mut self, sql: &SqlSegment) {
        if !self.sql.is_empty() {
            self.sql.push(' ');
        }
        self.sql.push_str(&sql.query);
    }

    fn add_arg(&mut self, arg: &ArgSegment) {
        if cfg!(feature = "postgres") {
            self.sql.push_str(&format!(" ${}", self.args.len() + 1));
        } else {
            self.sql.push_str(" ?");
        }
        self.args.push(arg.argument.clone());
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

    fn to_query(&self, testing: bool, branch_counter: &mut usize) -> TokenStream {
        let condition = &self.condition;
        let then = self.then.to_query(testing, branch_counter);
        let or_else = self.or_else.to_query(testing, branch_counter);
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

    fn add_arg(&mut self, arg: &ArgSegment) {
        self.then.add_arg(arg);
        self.or_else.add_arg(arg);
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

    fn to_query(&self, testing: bool, branch_counter: &mut usize) -> TokenStream {
        let expr = &self.expr;
        let arms = self
            .arms
            .iter()
            .map(|arm| arm.to_query(testing, branch_counter));
        quote! {
            match #expr { #(#arms,)* }
        }
    }

    fn add_sql(&mut self, sql: &SqlSegment) {
        for arm in &mut self.arms {
            arm.add_sql(sql);
        }
    }

    fn add_arg(&mut self, arg: &ArgSegment) {
        for arm in &mut self.arms {
            arm.add_arg(arg)
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

    fn to_query(&self, testing: bool, branch_counter: &mut usize) -> TokenStream {
        let pat = &self.pattern;
        let inner = self.inner.to_query(testing, branch_counter);
        quote! {
            #pat => #inner
        }
    }

    fn add_sql(&mut self, sql: &SqlSegment) {
        self.inner.add_sql(sql);
    }

    fn add_arg(&mut self, arg: &ArgSegment) {
        self.inner.add_arg(arg);
    }
}