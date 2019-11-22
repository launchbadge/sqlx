#[cfg(feature = "macros")]
use proc_macro_hack::proc_macro_hack;

#[doc(inline)]
pub use sqlx_core::*;

#[cfg(feature = "macros")]
#[proc_macro_hack(fake_call_site)]
pub use sqlx_macros::query;
