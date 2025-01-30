use quote::{__private::TokenStream, format_ident, quote};
use syn::{DeriveInput, Field};
use crate::derives::orm::helper::{db_pool_token, get_field_name};

pub fn generate_query(
    table_name: &str,
    columns: &str,
    input_struct: &DeriveInput,
    by_fields: &[&Field],
) -> TokenStream {
    let struct_name = &input_struct.ident;
    let struct_visibility = &input_struct.vis;
    let trait_ident = format_ident!("{}QueryTrait", struct_name);
    let builder_struct_ident = format_ident!("{}QueryBuilder", struct_name);
    let (pool, _) = db_pool_token();

    let impl_tokens: Vec<TokenStream> = by_fields.iter().filter_map(|field| {
        let field_ident = field.ident.as_ref().unwrap();
        let field_name = get_field_name(field);
        let where_fn = format_ident!("where_{}", field_ident);
        let order_by_fn = format_ident!("order_by_{}", field_ident);
        let group_by_fn = format_ident!("group_by_{}", field_ident);

        let code = quote! {
            #struct_visibility fn #where_fn(mut self, obj: &#struct_name, where_stmt: op::WhereOp) -> #builder_struct_ident {
                let stmt = format!("{} {} {}", #field_name, where_stmt, obj.#field_ident).to_string();
                self.where_stmt.push(stmt);
                self
            }

            #struct_visibility fn #order_by_fn(mut self, order_by: op::OrderBy) -> #builder_struct_ident {
                let stmt = format!("{} {}", #field_name, order_by).to_string();
                self.order_by_stmt.push(stmt);
                self
            }

            #struct_visibility fn #group_by_fn(mut self) -> #builder_struct_ident {
                let stmt = format!("{}", #field_name).to_string();
                self.group_by_stmt.push(stmt);
                self
            }
        };
        Some(code)
    }).collect::<Vec<_>>();

    quote! {
        #struct_visibility trait #trait_ident {
            fn query() -> #builder_struct_ident;
        }

        impl #trait_ident for #struct_name {
            #struct_visibility fn query() -> #builder_struct_ident {
                #builder_struct_ident::default()
            }
        }

        #[derive(Default)]
        #struct_visibility struct #builder_struct_ident {
            where_stmt: Vec<String>,
            order_by_stmt: Vec<String>,
            group_by_stmt: Vec<String>,
            limit: Option<i64>,
            offset: Option<i64>,
        }

        impl #builder_struct_ident {
            #struct_visibility fn limit(mut self, limit: i64) -> #builder_struct_ident {
                self.limit = Some(limit);
                self
            }

            #struct_visibility fn offset(mut self, offset: i64) -> #builder_struct_ident {
                self.offset = Some(offset);
                self
            }

            #(#impl_tokens)*

            #struct_visibility async fn build(self, pool: &#pool) -> sqlx::Result<Vec<#struct_name>> {
                let where_stmt = match self.where_stmt.is_empty() {
                    true => "".to_string(),
                    false => format!("WHERE {}", self.where_stmt.join(" AND ")),
                };
                let from = #table_name;
                let order_by_stmt = match self.order_by_stmt.is_empty() {
                    true => "".to_string(),
                    false => format!("ORDER BY {}", self.order_by_stmt.join(",")),
                };
                let group_by_stmt = match self.group_by_stmt.is_empty() {
                    true => "".to_string(),
                    false => format!("GROUP BY {}", self.group_by_stmt.join(",")),
                };
                let limit = match self.limit {
                    None => "".to_string(),
                    Some(v) => format!("LIMIT {}", v),
                };
                let offset = match self.offset {
                    None => "".to_string(),
                    Some(v) => format!("OFFSET {}", v),
                };
                let sql = format!(
                    "SELECT {} FROM {} {} {} {} {} {}",
                    #columns, from, where_stmt, group_by_stmt, order_by_stmt, limit, offset
                );
                let sql = sql.trim();
                sqlx::query_as::<_, #struct_name>(sql).fetch_all(pool).await
            }
        }
    }
}
