use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::Ident;

pub fn generate_conditional_map(n: usize) -> TokenStream {
    let call_site = Span::call_site();
    let map_fns = (1..=n)
        .map(|i| format_ident!("F{}", i))
        .collect::<Vec<_>>();
    let args = (1..=n)
        .map(|i| format_ident!("A{}", i))
        .collect::<Vec<_>>();
    let variants = (1..=n)
        .map(|i| format_ident!("_{}", i))
        .collect::<Vec<_>>();
    let variant_declarations = (0..n).map(|i| {
        let variant = &variants[i];
        let map_fn = &map_fns[i];
        let args = &args[i];
        quote!(#variant(sqlx::query::Map<'q, DB, #map_fn, #args>))
    });

    quote! {
        #[doc(hidden)]
        pub enum ConditionalMap<'q, DB, O, #(#map_fns,)* #(#args,)*>
        where
            DB: sqlx::Database,
            O: Send + Unpin,
            #(#map_fns: FnMut(DB::Row) -> sqlx::Result<O> + Send,)*
            #(#args: 'q + Send + sqlx::IntoArguments<'q, DB>,)*
        {
            #(#variant_declarations),*
        }
        impl<'q, DB, O, #(#map_fns,)* #(#args,)*> ConditionalMap<'q, DB, O, #(#map_fns,)* #(#args,)*>
        where
            DB: sqlx::Database,
            O: Send + Unpin,
            #(#map_fns: FnMut(DB::Row) -> sqlx::Result<O> + Send,)*
            #(#args: 'q + Send + sqlx::IntoArguments<'q, DB>,)*
        {
            pub fn fetch<'e, 'c: 'e, E>(self, executor: E) -> ormx::exports::futures::stream::BoxStream<'e, sqlx::Result<O>>
            where
                'q: 'e,
                E: 'e + sqlx::Executor<'c, Database = DB>,
                DB: 'e,
                O: 'e,
                #(#map_fns: 'e,)*
            {
                match self { #(
                    Self::#variants(x) => x.fetch(executor)
                ),* }
            }
            pub async fn fetch_all<'e, 'c: 'e, E>(self, executor: E) -> sqlx::Result<Vec<O>>
            where
                'q: 'e,
                DB: 'e,
                E: 'e + sqlx::Executor<'c, Database = DB>,
                O: 'e
            {
               match self { #(
                    Self::#variants(x) => x.fetch_all(executor).await
               ),* }
            }
            pub async fn fetch_one<'e, 'c: 'e, E>(self, executor: E) -> sqlx::Result<O>
            where
                'q: 'e,
                E: 'e + sqlx::Executor<'c, Database = DB>,
                DB: 'e,
                O: 'e,
            {
                match self { #(
                    Self::#variants(x) => x.fetch_one(executor).await
                ),* }
            }
            pub async fn fetch_optional<'e, 'c: 'e, E>(self, executor: E) -> sqlx::Result<Option<O>>
            where
                'q: 'e,
                E: 'e + sqlx::Executor<'c, Database = DB>,
                DB: 'e,
                O: 'e,
            {
                match self { #(
                    Self::#variants(x) => x.fetch_optional(executor).await
                ),* }
            }
        }
    }
}
