use sqlx::Backend;

pub trait BackendExt: Backend {
    const BACKEND_PATH: &'static str;

    fn quotable_path() -> syn::Path {
        syn::parse_str(Self::BACKEND_PATH).unwrap()
    }

    fn param_type_for_id(id: &Self::TypeId) -> Option<&'static str>;

    fn return_type_for_id(id: &Self::TypeId) -> Option<&'static str>;
}

macro_rules! impl_backend_ext {
    ($backend:path { $($(#[$meta:meta])? $ty:ty $(| $borrowed:ty)?),* }) => {
        impl $crate::backend::BackendExt for $backend {
            const BACKEND_PATH: &'static str = stringify!($backend);

            fn param_type_for_id(id: &Self::TypeId) -> Option<&'static str> {
                use sqlx::types::TypeMetadata;

                match () {
                    $(
                        // `if` statements cannot have attributes but these can
                        $(#[$meta])?
                        _ if <$backend as sqlx::types::HasSqlType<$ty>>::metadata().type_id_eq(id) => Some(borrowed_ty!($ty $(, $borrowed)?)),
                    )*
                    _ => None
                }
            }

            fn return_type_for_id(id: &Self::TypeId) -> Option<&'static str> {
                use sqlx::types::TypeMetadata;

                match () {
                    $(
                        $(#[$meta])?
                        _ if <$backend as sqlx::types::HasSqlType<$ty>>::metadata().type_id_eq(id) => return Some(stringify!($ty)),
                    )*
                    _ => None
                }
            }
        }
    }
}

macro_rules! borrowed_ty {
    ($ty:ty, $borrowed:ty) => {
        stringify!($borrowed)
    };
    ($ty:ty) => {
        stringify!($ty)
    };
}

#[cfg(feature = "postgres")]
mod postgres;

#[cfg(feature = "mariadb")]
mod mariadb;
