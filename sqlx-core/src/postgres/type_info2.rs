#![allow(dead_code)]

//! Postgres type informations
//!
//! SQLx has a hierarchy of Postgres types based on how much is known about a type:
//!
//! - `PgBuiltinType` represents Postgres types from the default catalog. Those
//!   are always defined and have stable "type OID" value. SQLx contains static
//!   list defining these types and always has full knowledge about them.
//! - `PgType` is a runtime version of `PgBuiltinType`. It enables support for
//!   for custom types that are not fully known at compile time.It supports the
//!   same information as a `PgBuiltinType`, except resolved at runtime:
//!   "type oid", name and kind. Resolving
//! - `LazyPgType`

use crate::ext::ustr::UStr;
use crate::postgres::catalog::PgTypeRef;
use crate::postgres::type_info as type_info1;
use crate::postgres::types::Oid;
use core::convert::TryFrom;
use core::iter::Once;
use std::fmt::Debug;
use std::ops::Deref;

/// Type alias to make it clearer when an OID actually refers to a type (a row
/// in `pg_catalog.pg_type`, as opposed to any other kind of Postgres object).
///
/// We may eventually replace it with a newtype if needed/wanted.
///
/// See <https://www.postgresql.org/docs/current/catalog-pg-type.html>.
pub type PgTypeOid = Oid;

/// Canonical local name of a Postgres type.
///
/// This corresponds to values in the column `typname` from the
/// `pg_catalog.pg_type` table.
/// Note: `typname` is not unique. Use `PgFullTypeName` for uniqueness.
///
/// This is usually a valid lowercase identifier.
///
/// Examples:
/// - `int8`: the `INT8` primitive type
/// - `_int8`: array of `INT8`
/// - `int8range`: range of `INT8`
/// - `_int8range`: array of range of `INT8`
///
/// See <https://www.postgresql.org/docs/current/catalog-pg-type.html>.
pub struct PgTypeLocalName<Str: Deref<Target = str> = UStr>(pub Str);

/// Canonical full name of a Postgres type.
///
/// This corresponds to the pair `(typnamespace, typname)` from the
/// `pg_catalog.pg_type` table. This pair is unique.
///
/// See <https://www.postgresql.org/docs/current/catalog-pg-namespace.html>.
pub struct PgTypeFullName<Str: Deref<Target = str> = UStr> {
    /// Namespace of the type
    // TODO: Add actual support for namespaces!
    namespace: (),
    /// Local name of the type
    name: PgTypeLocalName<Str>,
}

/// A resolved Postgres type
///
/// In SQLx versions before `0.6`, it was called `PgCustomType`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct PgType<TyDep> {
    pub(crate) oid: PgTypeOid,
    pub(crate) name: UStr,
    pub(crate) kind: PgTypeKind<TyDep>,
}

/// Type of a Postgres type.
///
/// See:
/// - <https://www.postgresql.org/docs/13/catalog-pg-type.html>
/// - <https://www.postgresql.org/docs/13/catalog-pg-type.html#CATALOG-TYPCATEGORY-TABLE>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
pub(crate) enum PgTypeKind<TyDep, Composite = OwningPgCompositeKind<TyDep>> {
    /// `b` in `pg_type.typtype`.
    // TODO: Rename to `Base` for consistency with Postgres.
    Simple,
    /// `p` in `pg_type.typtype`.
    Pseudo,
    /// `d` in `pg_type.typtype`.
    ///
    /// With the wrapped type.
    Domain(TyDep),
    /// `c` in `pg_type.typtype`.
    ///
    /// With the list of fields.
    Composite(Composite),
    /// `A` in `pg_type.typcategory` (represent arrays as a first-class kind)
    ///
    /// With the element type.
    Array(TyDep),
    /// `e` in `pg_type.typtype`.
    ///
    /// With the variant list.
    Enum(Box<[String]>),
    /// `r` in `pg_type.typtype`.
    ///
    /// With item type.
    Range(TyDep),
}

impl<TyDep> PgTypeKind<TyDep> {
    pub(crate) fn composite(fields: impl Into<Box<[(String, TyDep)]>>) -> Self {
        Self::Composite(OwningPgCompositeKind {
            fields: fields.into(),
        })
    }
}

/// Postgres composite kind details, owning its fields
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct OwningPgCompositeKind<TyDep> {
    /// Field list
    fields: Box<[(String, TyDep)]>,
}

impl<TyDep> OwningPgCompositeKind<TyDep> {
    pub(crate) fn fields(
        &self,
    ) -> impl Iterator<Item = (&str, &TyDep)> + DoubleEndedIterator + ExactSizeIterator {
        self.fields.iter().map(|(name, ty)| (name.as_str(), ty))
    }
}

impl<TyDep> PgTypeKind<TyDep> {
    pub(crate) fn map_dependencies<R, F>(self, mut f: F) -> PgTypeKind<R>
    where
        F: FnMut(TyDep) -> R,
    {
        match self {
            Self::Simple => PgTypeKind::Simple,
            Self::Pseudo => PgTypeKind::Pseudo,
            Self::Domain(wrapped) => PgTypeKind::Domain(f(wrapped)),
            Self::Composite(composite) => PgTypeKind::Composite(OwningPgCompositeKind {
                fields: composite
                    .fields
                    .into_vec()
                    .into_iter()
                    .map(|(k, t)| (k, f(t)))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            }),
            Self::Array(elem) => PgTypeKind::Array(f(elem)),
            Self::Enum(variants) => PgTypeKind::Enum(variants),
            Self::Range(item) => PgTypeKind::Range(f(item)),
        }
    }

    pub(crate) fn type_dependencies(&self) -> PgTypeDeps<'_, TyDep> {
        match self {
            Self::Simple => PgTypeDeps::Zero,
            Self::Pseudo => PgTypeDeps::Zero,
            Self::Enum(_) => PgTypeDeps::Zero,
            Self::Domain(wrapped) => PgTypeDeps::One(core::iter::once(wrapped)),
            Self::Array(elem) => PgTypeDeps::One(core::iter::once(elem)),
            Self::Range(item) => PgTypeDeps::One(core::iter::once(item)),
            Self::Composite(composite) => PgTypeDeps::Composite((&*composite.fields).into_iter()),
        }
    }
}

/// Iterator over the direct type dependencies of of a `PgTypeKind` (or `PgType`).
///
/// This iterator is guaranteed to be finite, but it may yield duplicates.
#[derive(Debug, Clone)]
pub(crate) enum PgTypeDeps<'a, TyDep> {
    Zero,
    One(Once<&'a TyDep>),
    Composite(core::slice::Iter<'a, (String, TyDep)>),
}

impl<'a, TyDep> Iterator for PgTypeDeps<'a, TyDep> {
    type Item = &'a TyDep;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Zero => None,
            Self::One(inner) => inner.next(),
            Self::Composite(fields) => fields.next().map(|(_, ty)| ty),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Zero => (0, Some(0)),
            Self::One(inner) => inner.size_hint(),
            Self::Composite(fields) => fields.size_hint(),
        }
    }
}

impl<'a, TyDep> DoubleEndedIterator for PgTypeDeps<'a, TyDep> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            Self::Zero => None,
            Self::One(inner) => inner.next_back(),
            Self::Composite(fields) => fields.next_back().map(|(_, ty)| ty),
        }
    }
}

impl<'a, TyDep> ExactSizeIterator for PgTypeDeps<'a, TyDep> {}

/// Represents a builtin Postgres type or pseudo-type.
///
/// SQLx has native understanding of builtin types. They can be used without
/// querying the database for metadata.
///
/// See <https://www.postgresql.org/docs/14/datatype.html>
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
#[repr(u32)]
pub(crate) enum PgBuiltinType {
    Bool,
    Bytea,
    Char,
    Name,
    Int8,
    Int2,
    Int4,
    Text,
    Oid,
    Json,
    JsonArray,
    Point,
    Lseg,
    Path,
    Box,
    Polygon,
    Line,
    LineArray,
    Cidr,
    CidrArray,
    Float4,
    Float8,
    /// Identifies a not-yet-resolved type, e.g., of an undecorated string literal.
    Unknown,
    Circle,
    CircleArray,
    Macaddr8,
    Macaddr8Array,
    Macaddr,
    Inet,
    BoolArray,
    ByteaArray,
    CharArray,
    NameArray,
    Int2Array,
    Int4Array,
    TextArray,
    BpcharArray,
    VarcharArray,
    Int8Array,
    PointArray,
    LsegArray,
    PathArray,
    BoxArray,
    Float4Array,
    Float8Array,
    PolygonArray,
    OidArray,
    MacaddrArray,
    InetArray,
    Bpchar,
    Varchar,
    Date,
    Time,
    Timestamp,
    TimestampArray,
    DateArray,
    TimeArray,
    Timestamptz,
    TimestamptzArray,
    Interval,
    IntervalArray,
    NumericArray,
    Timetz,
    TimetzArray,
    Bit,
    BitArray,
    Varbit,
    VarbitArray,
    Numeric,
    Record,
    RecordArray,
    Uuid,
    UuidArray,
    Jsonb,
    JsonbArray,
    Int4Range,
    Int4RangeArray,
    NumRange,
    NumRangeArray,
    TsRange,
    TsRangeArray,
    TstzRange,
    TstzRangeArray,
    DateRange,
    DateRangeArray,
    Int8Range,
    Int8RangeArray,
    Jsonpath,
    JsonpathArray,
    Money,
    MoneyArray,
    /// Indicates that a function returns no value.
    Void,
}

macro_rules! template_to_pg_kind {
    ($ty_dep:ident, (PgTypeKind::Simple)) => {
        PgTypeKind::Simple
    };
    ($ty_dep:ident, (PgTypeKind::Pseudo)) => {
        PgTypeKind::Pseudo
    };
    ($ty_dep:ident, (PgTypeKind::Array(PgBuiltinType::$dep:ident))) => {
        PgTypeKind::Array($ty_dep::$dep)
    };
    ($ty_dep:ident, (PgTypeKind::Range(PgBuiltinType::$dep:ident))) => {
        PgTypeKind::Range($ty_dep::$dep)
    };
}

macro_rules! builtin_kind_to_legacy_kind {
    (PgTypeKind::Simple) => {
        &type_info1::PgTypeKind::Simple
    };
    (PgTypeKind::Pseudo) => {
        &type_info1::PgTypeKind::Pseudo
    };
    (PgTypeKind::Array(PgBuiltinType::$ident:ident)) => {
        &type_info1::PgTypeKind::Array(type_info1::PgTypeInfo::$ident)
    };
    (PgTypeKind::Range(PgBuiltinType::$ident:ident)) => {
        &type_info1::PgTypeKind::Range(type_info1::PgTypeInfo::$ident)
    };
}

macro_rules! impl_builtin {
    ($(($ident:ident, $upper:ident, $oid:literal, $display_name:literal, $name:literal, $kind:tt $(,)?)),* $(,)?) => {
        impl PgBuiltinType {
            pub(crate) const fn try_from_oid(oid: PgTypeOid) -> Option<Self> {
                match oid.to_u32() {
                    $($oid => Some(Self::$ident),)*
                    _ => None,
                }
            }

            pub(crate) fn try_from_name(name: &str) -> Option<Self> {
                match name {
                    $($name => Some(Self::$ident),)*
                    _ => None,
                }
            }

            pub(crate) const fn oid(self) -> PgTypeOid {
                self.const_into::<PgTypeOid>()
            }

            pub(crate) const fn display_name(self) -> &'static str {
                match self {
                    $(Self::$ident => $display_name,)*
                }
            }

            pub(crate) const fn name(self) -> &'static str {
                self.const_into::<&'static str>()
            }

            pub(crate) const fn kind(self) -> PgTypeKind<PgBuiltinType> {
                match self {
                    $(Self::$ident => $kind,)*
                }
            }

            pub(crate) const fn legacy_kind(self) -> &'static type_info1::PgTypeKind {
                match self {
                    $(Self::$ident => builtin_kind_to_legacy_kind! $kind ,)*
                }
            }

            pub(crate) const fn into_legacy_type(self) -> type_info1::PgType {
                match self {
                    $(Self::$ident => type_info1::PgType::$ident,)*
                }
            }

            pub(crate) const fn try_from_legacy_type<'a>(t: &'a type_info1::PgType) -> Result<Self, ()> {
                match t {
                    $(type_info1::PgType::$ident => Ok(Self::$ident),)*
                    _ => Err(()),
                }
            }

            pub(crate) const fn const_into<T: ConstFromPgBuiltinType>(self) -> T {
                match self {
                    $(Self::$ident => T::$upper,)*
                }
            }

            pub(crate) const fn into_static_pg_type_with_ref(self) -> &'static PgType<PgTypeRef> {
                match self {
                    $(Self::$ident => {const PG_TYPE: &'static PgType<PgTypeRef> = &PgType::$upper; PG_TYPE },)*
                }
            }

            pub(crate) const fn into_static_pg_type_with_oid(self) -> &'static PgType<PgTypeOid> {
                match self {
                    $(Self::$ident => {const PG_TYPE: &'static PgType<PgTypeOid> = &PgType::$upper; PG_TYPE },)*
                }
            }

            pub(crate) fn iter() -> std::array::IntoIter<Self, 92> {
                [
                    $(Self::$ident,)*
                ].into_iter()
            }
        }

        pub(crate) trait ConstFromPgBuiltinType {
            $(const $upper: Self;)*
        }

        pub(crate) trait FromPgBuiltinType {
            fn from_pg_builtin_type(builtin: PgBuiltinType) -> Self;
        }

        impl<T: ConstFromPgBuiltinType> FromPgBuiltinType for T {
            fn from_pg_builtin_type(builtin: PgBuiltinType) -> Self {
                match builtin {
                    $(PgBuiltinType::$ident => Self::$upper,)*
                }
            }
        }

        impl ConstFromPgBuiltinType for PgTypeOid {
            $(const $upper: Self = PgTypeOid::from_u32($oid);)*
        }

        impl ConstFromPgBuiltinType for &'static str {
            $(const $upper: Self = $name;)*
        }

        impl ConstFromPgBuiltinType for PgTypeRef {
            $(const $upper: Self = PgTypeRef::Oid(PgTypeOid::from_u32($oid));)*
        }

        impl ConstFromPgBuiltinType for PgBuiltinType {
            $(const $upper: Self = PgBuiltinType::$ident;)*
        }

        impl<TyDep: ConstFromPgBuiltinType> ConstFromPgBuiltinType for PgTypeKind<TyDep> {
            $(const $upper: Self = template_to_pg_kind!(TyDep, $kind);)*
        }

        impl<TyDep: ConstFromPgBuiltinType> ConstFromPgBuiltinType for PgType<TyDep> {
            $(const $upper: Self = Self {
                oid: PgBuiltinType::$upper.oid(),
                name: UStr::Static(PgBuiltinType::$upper.name()),
                kind: PgTypeKind::<TyDep>::$upper,
            };)*
        }

        impl<TyDep: ConstFromPgBuiltinType> PgType<TyDep> {
            $(const $upper: Self = <Self as ConstFromPgBuiltinType>::$upper;)*
        }

        impl From<PgBuiltinType> for type_info1::PgType {
            fn from(t: PgBuiltinType) -> Self {
                t.into_legacy_type()
            }
        }

        impl<'a> TryFrom<&'a type_info1::PgType> for PgBuiltinType {
            type Error = ();

            fn try_from(t: &'a type_info1::PgType) -> Result<Self, Self::Error> {
                Self::try_from_legacy_type(t)
            }
        }
    };
}

impl PgBuiltinType {
    pub(crate) fn try_from_ref(ty_ref: &PgTypeRef) -> Option<Self> {
        match ty_ref {
            PgTypeRef::Oid(oid) => PgBuiltinType::try_from_oid(*oid),
            PgTypeRef::Name(name) => PgBuiltinType::try_from_name(name),
            PgTypeRef::OidAndName(oid, name) => match (
                PgBuiltinType::try_from_oid(*oid),
                PgBuiltinType::try_from_name(name),
            ) {
                (Some(a), Some(b)) if a == b => Some(a),
                _ => None,
            },
        }
    }
}

// DEVELOPER PRO TIP: find builtin type OIDs easily by grepping this file
// https://github.com/postgres/postgres/blob/master/src/include/catalog/pg_type.dat
//
// If you have Postgres running locally you can also try
// SELECT oid, typarray FROM pg_type where typname = '<type name>'

#[rustfmt::skip]
impl_builtin![
//  (VariantName,      ConstName,          oid, display_name,    name,           (kind))
    (Bool,             BOOL,                16, "BOOL",          "bool",         (PgTypeKind::Simple)),
    (Bytea,            BYTEA,               17, "BYTEA",         "bytea",        (PgTypeKind::Simple)),
    (Char,             CHAR,                18, "\"CHAR\"",      "char",         (PgTypeKind::Simple)),
    (Name,             NAME,                19, "NAME",          "name",         (PgTypeKind::Simple)),
    (Int8,             INT8,                20, "INT8",          "int8",         (PgTypeKind::Simple)),
    (Int2,             INT2,                21, "INT2",          "int2",         (PgTypeKind::Simple)),
    (Int4,             INT4,                23, "INT4",          "int4",         (PgTypeKind::Simple)),
    (Text,             TEXT,                25, "TEXT",          "text",         (PgTypeKind::Simple)),
    (Oid,              OID,                 26, "OID",           "oid",          (PgTypeKind::Simple)),
    (Json,             JSON,               114, "JSON",          "json",         (PgTypeKind::Simple)),
    (JsonArray,        JSON_ARRAY,         199, "JSON[]",        "_json",        (PgTypeKind::Array(PgBuiltinType::JSON))),
    (Point,            POINT,              600, "POINT",         "point",        (PgTypeKind::Simple)),
    (Lseg,             LSEG,               601, "LSEG",          "lseg",         (PgTypeKind::Simple)),
    (Path,             PATH,               602, "PATH",          "path",         (PgTypeKind::Simple)),
    (Box,              BOX,                603, "BOX",           "box",          (PgTypeKind::Simple)),
    (Polygon,          POLYGON,            604, "POLYGON",       "polygon",      (PgTypeKind::Simple)),
    (Line,             LINE,               628, "LINE",          "line",         (PgTypeKind::Simple)),
    (LineArray,        LINE_ARRAY,         629, "LINE[]",        "_line",        (PgTypeKind::Array(PgBuiltinType::LINE))),
    (Cidr,             CIDR,               650, "CIDR",          "cidr",         (PgTypeKind::Simple)),
    (CidrArray,        CIDR_ARRAY,         651, "CIDR[]",        "_cidr",        (PgTypeKind::Array(PgBuiltinType::CIDR))),
    (Float4,           FLOAT4,             700, "FLOAT4",        "float4",       (PgTypeKind::Simple)),
    (Float8,           FLOAT8,             701, "FLOAT8",        "float8",       (PgTypeKind::Simple)),
    (Unknown,          UNKNOWN,            705, "UNKNOWN",       "unknown",      (PgTypeKind::Simple)),
    (Circle,           CIRCLE,             718, "CIRCLE",        "circle",       (PgTypeKind::Simple)),
    (CircleArray,      CIRCLE_ARRAY,       719, "CIRCLE[]",      "_circle",      (PgTypeKind::Array(PgBuiltinType::CIRCLE))),
    (Macaddr8,         MACADDR8,           774, "MACADDR8",      "macaddr8",     (PgTypeKind::Simple)),
    (Macaddr8Array,    MACADDR8_ARRAY,     775, "MACADDR8[]",    "_macaddr8",    (PgTypeKind::Array(PgBuiltinType::MACADDR8))),
    (Money,            MONEY,              790, "MONEY",         "money",        (PgTypeKind::Simple)),
    (MoneyArray,       MONEY_ARRAY,        791, "MONEY[]",       "_money",       (PgTypeKind::Array(PgBuiltinType::MONEY))),
    (Macaddr,          MACADDR,            829, "MACADDR",       "macaddr",      (PgTypeKind::Simple)),
    (Inet,             INET,               869, "INET",          "inet",         (PgTypeKind::Simple)),
    (BoolArray,        BOOL_ARRAY,        1000, "BOOL[]",        "_bool",        (PgTypeKind::Array(PgBuiltinType::BOOL))),
    (ByteaArray,       BYTE_ARRAY,        1001, "BYTEA[]",       "_bytea",       (PgTypeKind::Array(PgBuiltinType::BYTEA))),
    (CharArray,        CHAR_ARRAY,        1002, "\"CHAR\"[]",    "_char",        (PgTypeKind::Array(PgBuiltinType::CHAR))),
    (NameArray,        NAME_ARRAY,        1003, "NAME[]",        "_name",        (PgTypeKind::Array(PgBuiltinType::NAME))),
    (Int2Array,        INT2_ARRAY,        1005, "INT2[]",        "_int2",        (PgTypeKind::Array(PgBuiltinType::INT2))),
    (Int4Array,        INT4_ARRAY,        1007, "INT4[]",        "_int4",        (PgTypeKind::Array(PgBuiltinType::INT4))),
    (TextArray,        TEXT_ARRAY,        1009, "TEXT[]",        "_text",        (PgTypeKind::Array(PgBuiltinType::TEXT))),
    (BpcharArray,      BPCHAR_ARRAY,      1014, "CHAR[]",        "_bpchar",      (PgTypeKind::Array(PgBuiltinType::BPCHAR))),
    (VarcharArray,     VARCHAR_ARRAY,     1015, "VARCHAR[]",     "_varchar",     (PgTypeKind::Array(PgBuiltinType::VARCHAR))),
    (Int8Array,        INT8_ARRAY,        1016, "INT8[]",        "_int8",        (PgTypeKind::Array(PgBuiltinType::INT8))),
    (PointArray,       POINT_ARRAY,       1017, "POINT[]",       "_point",       (PgTypeKind::Array(PgBuiltinType::POINT))),
    (LsegArray,        LSEG_ARRAY,        1018, "LSEG[]",        "_lseg",        (PgTypeKind::Array(PgBuiltinType::LSEG))),
    (PathArray,        PATH_ARRAY,        1019, "PATH[]",        "_path",        (PgTypeKind::Array(PgBuiltinType::PATH))),
    (BoxArray,         BOX_ARRAY,         1020, "BOX[]",         "_box",         (PgTypeKind::Array(PgBuiltinType::BOX))),
    (Float4Array,      FLOAT4_ARRAY,      1021, "FLOAT4[]",      "_float4",      (PgTypeKind::Array(PgBuiltinType::FLOAT4))),
    (Float8Array,      FLOAT8_ARRAY,      1022, "FLOAT8[]",      "_float8",      (PgTypeKind::Array(PgBuiltinType::FLOAT8))),
    (PolygonArray,     POLYGON_ARRAY,     1027, "POLYGON[]",     "_polygon",     (PgTypeKind::Array(PgBuiltinType::POLYGON))),
    (OidArray,         OID_ARRAY,         1028, "OID[]",         "_oid",         (PgTypeKind::Array(PgBuiltinType::OID))),
    (MacaddrArray,     MACADDR_ARRAY,     1040, "MACADDR[]",     "_macaddr",     (PgTypeKind::Array(PgBuiltinType::MACADDR))),
    (InetArray,        INET_ARRAY,        1041, "INET[]",        "_inet",        (PgTypeKind::Array(PgBuiltinType::INET))),
    (Bpchar,           BPCHAR,            1042, "CHAR",          "bpchar",       (PgTypeKind::Simple)),
    (Varchar,          VARCHAR,           1043, "VARCHAR",       "varchar",      (PgTypeKind::Simple)),
    (Date,             DATE,              1082, "DATE",          "date",         (PgTypeKind::Simple)),
    (Time,             TIME,              1083, "TIME",          "time",         (PgTypeKind::Simple)),
    (Timestamp,        TIMESTAMP,         1114, "TIMESTAMP",     "timestamp",    (PgTypeKind::Simple)),
    (TimestampArray,   TIMESTAMP_ARRAY,   1115, "TIMESTAMP[]",   "_timestamp",   (PgTypeKind::Array(PgBuiltinType::TIMESTAMP))),
    (DateArray,        DATE_ARRAY,        1182, "DATE[]",        "_date",        (PgTypeKind::Array(PgBuiltinType::DATE))),
    (TimeArray,        TIME_ARRAY,        1183, "TIME[]",        "_time",        (PgTypeKind::Array(PgBuiltinType::TIME))),
    (Timestamptz,      TIMESTAMPTZ,       1184, "TIMESTAMPTZ",   "timestamptz",  (PgTypeKind::Simple)),
    (TimestamptzArray, TIMESTAMPTZ_ARRAY, 1185, "TIMESTAMPTZ[]", "_timestamptz", (PgTypeKind::Array(PgBuiltinType::TIMESTAMPTZ))),
    (Interval,         INTERVAL,          1186, "INTERVAL",      "interval",     (PgTypeKind::Simple)),
    (IntervalArray,    INTERVAL_ARRAY,    1187, "INTERVAL[]",    "_interval",    (PgTypeKind::Array(PgBuiltinType::INTERVAL))),
    (NumericArray,     NUMERIC_ARRAY,     1231, "NUMERIC[]",     "_numeric",     (PgTypeKind::Array(PgBuiltinType::NUMERIC))),
    (Timetz,           TIMETZ,            1266, "TIMETZ",        "timetz",       (PgTypeKind::Simple)),
    (TimetzArray,      TIMETZ_ARRAY,      1270, "TIMETZ[]",      "_timetz",      (PgTypeKind::Array(PgBuiltinType::TIMETZ))),
    (Bit,              BIT,               1560, "BIT",           "bit",          (PgTypeKind::Simple)),
    (BitArray,         BIT_ARRAY,         1561, "BIT[]",         "_bit",         (PgTypeKind::Array(PgBuiltinType::BIT))),
    (Varbit,           VARBIT,            1562, "VARBIT",        "varbit",       (PgTypeKind::Simple)),
    (VarbitArray,      VARBIT_ARRAY,      1563, "VARBIT[]",      "_varbit",      (PgTypeKind::Array(PgBuiltinType::VARBIT))),
    (Numeric,          NUMERIC,           1700, "NUMERIC",       "numeric",      (PgTypeKind::Simple)),
    (Void,             VOID,              2278, "VOID",          "void",         (PgTypeKind::Pseudo)),
    (Record,           RECORD,            2249, "RECORD",        "record",       (PgTypeKind::Simple)),
    (RecordArray,      RECORD_ARRAY,      2287, "RECORD[]",      "_record",      (PgTypeKind::Array(PgBuiltinType::RECORD))),
    (Uuid,             UUID,              2950, "UUID",          "uuid",         (PgTypeKind::Simple)),
    (UuidArray,        UUID_ARRAY,        2951, "UUID[]",        "_uuid",        (PgTypeKind::Array(PgBuiltinType::UUID))),
    (Jsonb,            JSONB,             3802, "JSONB",         "jsonb",        (PgTypeKind::Simple)),
    (JsonbArray,       JSONB_ARRAY,       3807, "JSONB[]",       "_jsonb",       (PgTypeKind::Array(PgBuiltinType::JSONB))),
    (Int4Range,        INT4_RANGE,        3904, "INT4RANGE",     "int4range",    (PgTypeKind::Range(PgBuiltinType::INT4))),
    (Int4RangeArray,   INT4_RANGE_ARRAY,  3905, "INT4RANGE[]",   "_int4range",   (PgTypeKind::Array(PgBuiltinType::INT4_RANGE))),
    (NumRange,         NUM_RANGE,         3906, "NUMRANGE",      "numrange",     (PgTypeKind::Range(PgBuiltinType::NUMERIC))),
    (NumRangeArray,    NUM_RANGE_ARRAY,   3907, "NUMRANGE[]",    "_numrange",    (PgTypeKind::Array(PgBuiltinType::NUM_RANGE))),
    (TsRange,          TS_RANGE,          3908, "TSRANGE",       "tsrange",      (PgTypeKind::Range(PgBuiltinType::TIMESTAMP))),
    (TsRangeArray,     TS_RANGE_ARRAY,    3909, "TSRANGE[]",     "_tsrange",     (PgTypeKind::Array(PgBuiltinType::TS_RANGE))),
    (TstzRange,        TSTZ_RANGE,        3910, "TSTZRANGE",     "tstzrange",    (PgTypeKind::Range(PgBuiltinType::TIMESTAMPTZ))),
    (TstzRangeArray,   TSTZ_RANGE_ARRAY,  3911, "TSTZRANGE[]",   "_tstzrange",   (PgTypeKind::Array(PgBuiltinType::TSTZ_RANGE))),
    (DateRange,        DATE_RANGE,        3912, "DATERANGE",     "daterange",    (PgTypeKind::Range(PgBuiltinType::DATE))),
    (DateRangeArray,   DATE_RANGE_ARRAY,  3913, "DATERANGE[]",   "_daterange",   (PgTypeKind::Array(PgBuiltinType::DATE_RANGE))),
    (Int8Range,        INT8_RANGE,        3926, "INT8RANGE",     "int8range",    (PgTypeKind::Range(PgBuiltinType::INT8))),
    (Int8RangeArray,   INT8_RANGE_ARRAY,  3927, "INT8RANGE[]",   "_int8range",   (PgTypeKind::Array(PgBuiltinType::INT8_RANGE))),
    (Jsonpath,         JSONPATH,          4072, "JSONPATH",      "jsonpath",     (PgTypeKind::Simple)),
    (JsonpathArray,    JSONPATH_ARRAY,    4073, "JSONPATH[]",    "_jsonpath",    (PgTypeKind::Array(PgBuiltinType::JSONPATH))),
];

impl<TyDep> PgType<TyDep> {
    pub(crate) fn oid(&self) -> PgTypeOid {
        self.oid
    }

    pub(crate) fn name(&self) -> UStr {
        self.name.clone()
    }

    pub(crate) fn map_dependencies<R, F>(self, f: F) -> PgType<R>
    where
        F: FnMut(TyDep) -> R,
    {
        PgType {
            oid: self.oid,
            name: self.name,
            kind: self.kind.map_dependencies(f),
        }
    }

    pub(crate) fn type_dependencies(&self) -> PgTypeDeps<'_, TyDep> {
        self.kind.type_dependencies()
    }
}

impl PgType<PgBuiltinType> {
    pub(crate) fn from_builtin(builtin: PgBuiltinType) -> Self {
        Self {
            oid: builtin.oid(),
            name: builtin.name().into(),
            kind: builtin.kind(),
        }
    }
}

impl PgType<PgTypeRef> {
    pub(crate) fn from_builtin(builtin: PgBuiltinType) -> Self {
        PgType::<PgBuiltinType>::from_builtin(builtin)
            .map_dependencies(|builtin| PgTypeRef::Oid(builtin.oid()))
    }

    pub(crate) fn try_from_oid(oid: PgTypeOid) -> Option<Self> {
        PgBuiltinType::try_from_oid(oid).map(Self::from_builtin)
    }

    pub(crate) fn try_from_name(name: &str) -> Option<Self> {
        PgBuiltinType::try_from_name(name).map(Self::from_builtin)
    }
}

/// A type that may not be resolved yet.
#[derive(Debug, Clone)]
pub(crate) enum LazyPgType {
    /// The type is not resolved yet: only a reference is known.
    Ref(PgTypeRef),
    /// Resolved type information
    Resolved(PgType<PgTypeRef>),
}
