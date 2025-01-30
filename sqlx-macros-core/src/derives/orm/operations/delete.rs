use quote::{__private::TokenStream, format_ident, quote};
use syn::{DeriveInput, Field};
use crate::derives::orm::helper::{db_placeholder, db_pool_token, get_field_name};

pub fn generate_delete(
    table_name: &str,
    input_struct: &DeriveInput,
    pk_field: &Field,
) -> TokenStream {
    let struct_name = &input_struct.ident;
    let struct_visibility = &input_struct.vis;
    let trait_ident = format_ident!("{}DeleteTrait", struct_name);
    let (pool, _) = db_pool_token();

    // Primary key
    let pk_column = pk_field.ident.as_ref().unwrap();
    let pk_name = get_field_name(pk_field);
    let pk_placeholder = format!("{} = {}", pk_name, db_placeholder(1));

    quote! {
        #struct_visibility trait #trait_ident {
            #struct_visibility async fn delete(&self, pool: &#pool) -> sqlx::Result<()>;
        }

        impl #trait_ident for #struct_name {
            #struct_visibility async fn delete(&self, pool: &#pool) -> sqlx::Result<()> {
                let sql = format!("DELETE FROM {} WHERE {}", #table_name, #pk_placeholder);
                let _ = sqlx::query(&sql)
                .bind(&self.#pk_column)
                .execute(pool).await;
                Ok(())
            }
        }
    }
}
