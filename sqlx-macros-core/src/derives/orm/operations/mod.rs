mod by;
mod delete;
mod query;
mod save;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Field, Fields};
use crate::derives::orm::helper::{get_field_name, get_table_name, is_by, is_created_at, is_pk, is_readonly, is_transient, is_updated_at};

pub fn generate_orm(input: DeriveInput) -> TokenStream {
    let table_name = get_table_name(&input);
    let fields = match &input.data {
        Data::Struct(DataStruct {
                         fields: Fields::Named(fields),
                         ..
                     }) => &fields.named,
        _ => panic!("expected a struct with named fields"),
    };
    let mut by_fields: Vec<&Field> = vec![];
    let mut update_fields: Vec<&Field> = vec![];
    let mut insert_fields: Vec<&Field> = vec![];
    let mut struct_columns_vec: Vec<String> = vec![];
    let mut pk_field: Option<&Field> = None;
    let mut created_at_field: Option<&Field> = None;
    let mut updated_at_field: Option<&Field> = None;
    for field in fields.iter() {
        if !is_transient(field) {
            insert_fields.push(field);
            struct_columns_vec.push(format!("{}", get_field_name(field)));
            if is_pk(field) {
                pk_field = Some(field);
            }
            if is_created_at(field) {
                created_at_field = Some(field);
            }
            if is_updated_at(field) {
                updated_at_field = Some(field);
            }
            if is_by(field) || is_pk(field) {
                by_fields.push(field);
            }
            if !is_readonly(field) && !is_pk(field) {
                update_fields.push(field);
            }
        }
    }
    let struct_columns = struct_columns_vec.join(",");
    let pk_field = pk_field
        .unwrap_or_else(|| panic!("expected a primary key using #[orm(pk)] attribute on a field"));

    let by_code = by::generate_by(table_name.as_str(), &struct_columns, &input, &by_fields);
    let query_code =
        query::generate_query(table_name.as_str(), &struct_columns, &input, &by_fields);
    let delete_code = delete::generate_delete(table_name.as_str(), &input, pk_field);
    let upsert_code = save::generate_save(
        table_name.as_str(),
        &struct_columns,
        &input,
        pk_field,
        created_at_field,
        updated_at_field,
        &insert_fields,
        &update_fields,
    );

    TokenStream::from(quote! {
        #by_code
        #query_code
        #delete_code
        #upsert_code
    })
}
