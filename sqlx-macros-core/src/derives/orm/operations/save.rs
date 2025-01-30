use crate::derives::orm::helper::{
    create_insert_placeholders, create_update_placeholders, db_placeholder, db_pool_token,
    get_field_name, get_is_default_method, get_new_method,
};
use quote::{__private::TokenStream, format_ident, quote};
use syn::{DeriveInput, Field, Ident};

pub fn generate_save(
    table_name: &str,
    columns: &str,
    input_struct: &DeriveInput,
    pk_field: &Field,
    created_at_field: Option<&Field>,
    updated_at_field: Option<&Field>,
    insert_fields: &[&Field],
    update_fields: &[&Field],
) -> TokenStream {
    let struct_name = &input_struct.ident;
    let struct_visibility = &input_struct.vis;
    let trait_ident = format_ident!("{}SaveTrait", struct_name);
    let (pool, _) = db_pool_token();

    // prepare `insertable` fields
    let mut insert_columns_vec: Vec<String> = vec![];
    let mut insert_values: Vec<Option<&Ident>> = vec![];
    for field in insert_fields.iter() {
        insert_columns_vec.push(format!("{}", get_field_name(field)));
        insert_values.push(field.ident.as_ref());
    }
    let insert_columns = insert_columns_vec.join(",");
    let insert_value_placeholders = create_insert_placeholders(&insert_fields);

    // find `updatable` fields
    let update_value_placeholders = create_update_placeholders(update_fields);
    let update_values = update_fields
        .iter()
        .map(|field| &field.ident)
        .collect::<Vec<_>>();

    // Primary key
    let pk_column = pk_field.ident.as_ref().unwrap();
    let pk_name = get_field_name(pk_field);
    let pk_placeholder = format!("{} = {}", pk_name, db_placeholder(update_fields.len() + 1));
    let pk_new_method = get_new_method(pk_field);
    let pk_is_default_method = get_is_default_method(pk_field);

    let created_at_code = match created_at_field {
        None => quote! {},
        Some(field) => {
            let new_method = get_new_method(field);
            let column = field.ident.as_ref().unwrap();
            quote! {
                to_save.#column = #new_method
            }
        }
    };

    let updated_at_code = match updated_at_field {
        None => quote! {},
        Some(field) => {
            let new_method = get_new_method(field);
            let column = field.ident.as_ref().unwrap();
            quote! {
                to_save.#column = #new_method
            }
        }
    };

    quote! {
        #struct_visibility trait #trait_ident {
            #struct_visibility async fn save(&self, pool: &#pool) -> sqlx::Result<#struct_name>;
        }

        impl #trait_ident for #struct_name {
            #struct_visibility async fn save(&self, pool: &#pool) -> sqlx::Result<#struct_name> {
                let mut to_save = self.clone();
                #updated_at_code;
                match to_save.#pk_is_default_method {
                    true => {
                        to_save.#pk_column = #pk_new_method;
                        #created_at_code;
                        let sql = format!("INSERT INTO {} ({}) VALUES ({}) RETURNING {}", #table_name, #insert_columns, #insert_value_placeholders, #columns);
                        sqlx::query_as::<_, #struct_name>(&sql)
                        #(
                            .bind(&to_save.#insert_values)
                        )*
                        .fetch_one(pool).await
                    },
                    false => {
                        let sql = format!("UPDATE {} SET {} WHERE {} RETURNING {}", #table_name, #update_value_placeholders, #pk_placeholder, #columns);
                        sqlx::query_as::<_, #struct_name>(&sql)
                        #(
                            .bind(&self.#update_values)
                        )*
                        .bind(&self.#pk_column)
                        .fetch_one(pool).await
                    }
                }
            }
        }
    }
}
