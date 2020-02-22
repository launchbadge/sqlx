use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::Path;

use sqlx::describe::Describe;

use crate::database::DatabaseExt;

pub struct RustColumn {
    pub(super) ident: Ident,
    pub(super) type_: TokenStream,
}

pub fn columns_to_rust<DB: DatabaseExt>(describe: &Describe<DB>) -> crate::Result<Vec<RustColumn>> {
    describe
        .result_columns
        .iter()
        .enumerate()
        .map(|(i, column)| -> crate::Result<_> {
            let name = column
                .name
                .as_deref()
                .ok_or_else(|| format!("column at position {} must have a name", i))?;

            let ident = parse_ident(name)?;

            let type_ = <DB as DatabaseExt>::return_type_for_id(&column.type_info)
                .ok_or_else(|| format!("unknown type: {}", &column.type_info))?
                .parse::<TokenStream>()
                .unwrap();

            Ok(RustColumn { ident, type_ })
        })
        .collect::<crate::Result<Vec<_>>>()
}

pub fn quote_query_as<DB: DatabaseExt>(
    sql: &str,
    out_ty: &Path,
    bind_args: &Ident,
    columns: &[RustColumn],
) -> TokenStream {
    let instantiations = columns.iter().enumerate().map(
        |(
            i,
            &RustColumn {
                ref ident,
                ref type_,
                ..
            },
        )| { quote!( #ident: row.try_get::<#type_, _>(#i).try_unwrap_optional()? ) },
    );

    let db_path = DB::db_path();
    let row_path = DB::row_path();

    quote! {
        sqlx::query::<#db_path>(#sql).bind_all(#bind_args).map(|row: #row_path| {
            use sqlx::row::Row as _;
            use sqlx::result_ext::ResultExt as _;
            Ok(#out_ty { #(#instantiations),* })
        })
    }
}

fn parse_ident(name: &str) -> crate::Result<Ident> {
    // workaround for the following issue (it's semi-fixed but still spits out extra diagnostics)
    // https://github.com/dtolnay/syn/issues/749#issuecomment-575451318

    let is_valid_ident = name.chars().all(|c| c.is_alphanumeric() || c == '_');

    if is_valid_ident {
        let ident = String::from("r#") + name;
        if let Ok(ident) = syn::parse_str(&ident) {
            return Ok(ident);
        }
    }

    Err(format!("{:?} is not a valid Rust identifier", name).into())
}
