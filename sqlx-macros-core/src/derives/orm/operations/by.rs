use quote::{__private::TokenStream, format_ident, quote};
use syn::{DeriveInput, Field};
use crate::derives::orm::helper::{db_placeholder, db_pool_token, get_field_name, get_option_type};

pub fn generate_by(
    table_name: &str,
    columns: &str,
    input_struct: &DeriveInput,
    by_fields: &[&Field],
) -> TokenStream {
    let struct_name = &input_struct.ident;
    let struct_visibility = &input_struct.vis;
    let trait_ident = format_ident!("{}ByTrait", struct_name);

    let stream: Vec<(TokenStream, TokenStream)> = by_fields.iter().filter_map(|field| {
        let field_ident = field.ident.as_ref().unwrap();
        let (_, field_type) = get_option_type(&field.ty);
        let field_name = get_field_name(field);
        let by_fn = format_ident!("by_{}",field_ident);
        let placeholder = db_placeholder(1);
        let (pool, _) = db_pool_token();

        let trait_code = quote! {
            #struct_visibility async fn #by_fn(pool: &#pool, value: #field_type) -> sqlx::Result<Option<#struct_name>>;
        };

        let impl_code = quote! {
            #struct_visibility async fn #by_fn(pool: &#pool, value: #field_type) -> sqlx::Result<Option<#struct_name>> {
                let sql = format!("SELECT {} FROM {} WHERE {} = {}",#columns, #table_name, #field_name, #placeholder);
                sqlx::query_as::<_, Self>(&sql)
                .bind(value)
                .fetch_optional(pool).await
            }
        };
        Some((trait_code, impl_code))
    }).collect::<Vec<(_, _)>>();
    let (trait_tokens, impl_tokens): (Vec<TokenStream>, Vec<TokenStream>) =
        stream.into_iter().unzip();

    quote! {
        #struct_visibility trait #trait_ident {
            #(#trait_tokens)*
        }

        impl #trait_ident for #struct_name {
            #(#impl_tokens)*
        }
    }
}
