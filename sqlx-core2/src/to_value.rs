use crate::database::{Database, HasTypeId};
use crate::error::BoxStdError;

pub trait ToValue<DB: Database>: Send + Sync {
    fn accepts(&self, ty: &DB::TypeInfo) -> bool {
        // most database drivers have no parameter type information so there is no
        // point in tryin to implement this
        true
    }

    fn produces(&self) -> <DB as HasTypeId<'static>>::TypeId;

    fn to_value(&self, ty: &DB::TypeInfo, buf: &mut Vec<u8>) -> Result<(), BoxStdError>;

    #[doc(hidden)]
    #[inline]
    fn __type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}
