use sqlx_core::TypeInfo;

use crate::{PgTypeId, Postgres};

/// Provides information about a Postgres type.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(
    any(feature = "offline", feature = "serde"),
    derive(serde::Serialize, serde::Deserialize)
)]
#[allow(clippy::module_name_repetitions)]
pub struct PgTypeInfo(pub(crate) PgTypeId);

impl PgTypeInfo {
    /// Returns the unique identifier for this Postgres type.
    #[must_use]
    pub const fn id(&self) -> PgTypeId {
        self.0
    }

    /// Returns the name for this Postgres type.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.0.name()
    }
}

impl TypeInfo for PgTypeInfo {
    type Database = Postgres;

    fn id(&self) -> PgTypeId {
        self.id()
    }

    fn is_unknown(&self) -> bool {
        matches!(self.0, PgTypeId::UNKNOWN)
    }

    fn name(&self) -> &'static str {
        self.name()
    }
}
