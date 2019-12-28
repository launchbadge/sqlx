use sqlx::Database;

#[derive(PartialEq, Eq)]
pub enum ParamChecking {
    Strong,
    Weak,
}

pub trait DatabaseExt: Database {
    const DATABASE_PATH: &'static str;

    const PARAM_CHECKING: ParamChecking;

    fn quotable_path() -> syn::Path {
        syn::parse_str(Self::DATABASE_PATH).unwrap()
    }

    fn param_type_for_id(id: &Self::TypeId) -> Option<&'static str>;

    fn return_type_for_id(id: &Self::TypeId) -> Option<&'static str>;
}

macro_rules! impl_database_ext {
    ($database:path { $($(#[$meta:meta])? $ty:ty $(| $input:ty)?),*$(,)? }, ParamChecking::$param_checking:ident) => {
        impl $crate::database::DatabaseExt for $database {
            const DATABASE_PATH: &'static str = stringify!($database);
            const PARAM_CHECKING: $crate::database::ParamChecking = $crate::database::ParamChecking::$param_checking;

            fn param_type_for_id(id: &Self::TypeId) -> Option<&'static str> {
                match () {
                    $(
                        // `if` statements cannot have attributes but these can
                        $(#[$meta])?
                        _ if <$database as sqlx::types::HasSqlType<$ty>>::metadata().eq(id) => Some(input_ty!($ty $(, $input)?)),
                    )*
                    _ => None
                }
            }

            fn return_type_for_id(id: &Self::TypeId) -> Option<&'static str> {
                match () {
                    $(
                        $(#[$meta])?
                        _ if <$database as sqlx::types::HasSqlType<$ty>>::metadata().eq(id) => return Some(stringify!($ty)),
                    )*
                    _ => None
                }
            }
        }
    }
}

macro_rules! input_ty {
    ($ty:ty, $input:ty) => {
        stringify!($input)
    };
    ($ty:ty) => {
        stringify!($ty)
    };
}

#[cfg(feature = "postgres")]
mod postgres;

#[cfg(feature = "mysql")]
mod mysql;
