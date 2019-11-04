use proc_macro2::TokenStream;

pub fn map_param_type_oid(oid: u32) -> Option<TokenStream> {
    Some(match oid {
        16 => "bool",
        1000 => "&[bool]",
        25 => "&str",
        1009 => "&[&str]",
        21 => "i16",
        1005 => "&[i16]",
        23 => "i32",
        1007 => "&[i32]",
        20 => "i64",
        1016 => "&[i64]",
        700 => "f32",
        1021 => "&[f32]",
        701 => "f64",
        1022 => "&[f64]",
        2950 => "sqlx::Uuid",
        2951 => "&[sqlx::Uuid]",
        _ => return None
    }.parse().unwrap())
}

pub fn map_output_type_oid(oid: u32) -> crate::Result<TokenStream> {
    Ok(match oid {
        16 => "bool",
        1000 => "Vec<bool>",
        25 => "String",
        1009 => "Vec<String>",
        21 => "i16",
        1005 => "Vec<i16>",
        23 => "i32",
        1007 => "Vec<i32>",
        20 => "i64",
        1016 => "Vec<i64>",
        700 => "f32",
        1021 => "Vec<f32>",
        701 => "f64",
        1022 => "Vec<f64>",
        2950 => "sqlx::Uuid",
        2951 => "Vec<sqlx::Uuid>",
        _ => return Err(format!("unknown type ID: {}", oid).into())
    }.parse().unwrap())
}
