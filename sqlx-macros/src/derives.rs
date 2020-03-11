use quote::quote;
use syn::{parse_quote, Data, DataStruct, DeriveInput, Fields, FieldsUnnamed};

pub(crate) fn expand_derive_encode(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Unnamed(FieldsUnnamed { unnamed, .. }),
            ..
        }) if unnamed.len() == 1 => {
            let ident = &input.ident;
            let ty = &unnamed.first().unwrap().ty;

            // extract type generics
            let generics = &input.generics;
            let (_, ty_generics, _) = generics.split_for_impl();

            // add db type for impl generics & where clause
            let mut generics = generics.clone();
            generics.params.insert(0, parse_quote!(DB: sqlx::Database));
            generics
                .make_where_clause()
                .predicates
                .push(parse_quote!(#ty: sqlx::encode::Encode<DB>));
            let (impl_generics, _, where_clause) = generics.split_for_impl();

            Ok(quote!(
                impl #impl_generics sqlx::encode::Encode<DB> for #ident #ty_generics #where_clause {
                    fn encode(&self, buf: &mut std::vec::Vec<u8>) {
                        sqlx::encode::Encode::encode(&self.0, buf)
                    }
                    fn encode_nullable(&self, buf: &mut std::vec::Vec<u8>) -> sqlx::encode::IsNull {
                        sqlx::encode::Encode::encode_nullable(&self.0, buf)
                    }
                    fn size_hint(&self) -> usize {
                        sqlx::encode::Encode::size_hint(&self.0)
                    }
                }
            ))
        }
        _ => Err(syn::Error::new_spanned(
            input,
            "expected a tuple struct with a single field",
        )),
    }
}

pub(crate) fn expand_derive_decode(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Unnamed(FieldsUnnamed { unnamed, .. }),
            ..
        }) if unnamed.len() == 1 => {
            let ident = &input.ident;
            let ty = &unnamed.first().unwrap().ty;

            // extract type generics
            let generics = &input.generics;
            let (_, ty_generics, _) = generics.split_for_impl();

            let mut impls = Vec::new();

            if cfg!(feature = "postgres") {
                let mut generics = generics.clone();
                generics.params.insert(0, parse_quote!('de));
                generics
                    .make_where_clause()
                    .predicates
                    .push(parse_quote!(#ty: sqlx::decode::Decode<'de, sqlx::Postgres>));

                let (impl_generics, _, where_clause) = generics.split_for_impl();

                impls.push(quote!(
                    impl #impl_generics sqlx::decode::Decode<'de, sqlx::Postgres> for #ident #ty_generics #where_clause {
                        fn decode(value: <sqlx::Postgres as sqlx::HasRawValue<'de>>::RawValue) -> sqlx::Result<Self> {
                            <#ty as sqlx::decode::Decode<'de, sqlx::Postgres>>::decode(value).map(Self)
                        }
                    }
                ));
            }

            if cfg!(feature = "mysql") {
                let mut generics = generics.clone();
                generics.params.insert(0, parse_quote!('de));
                generics
                    .make_where_clause()
                    .predicates
                    .push(parse_quote!(#ty: sqlx::decode::Decode<'de, sqlx::MySql>));

                let (impl_generics, _, where_clause) = generics.split_for_impl();

                impls.push(quote!(
                    impl #impl_generics sqlx::decode::Decode<'de, sqlx::MySql> for #ident #ty_generics #where_clause {
                        fn decode(value: <sqlx::MySql as sqlx::HasRawValue<'de>>::RawValue) -> sqlx::Result<Self> {
                            <#ty as sqlx::decode::Decode<'de, sqlx::MySql>>::decode(value).map(Self)
                        }
                    }
                ));
            }

            // panic!("{}", q)
            Ok(quote!(#(#impls)*))
        }
        _ => Err(syn::Error::new_spanned(
            input,
            "expected a tuple struct with a single field",
        )),
    }
}
