use std::fmt::{self, Display, Formatter};

use crate::ext::ustr::UStr;
use crate::type_info::TypeInfo;

/// Type information for a PostgreSQL type.
#[derive(Debug, Clone, Eq)]
pub struct PgTypeInfo {
    pub(crate) id: Option<u32>,
    pub(crate) name: UStr,
}

impl PgTypeInfo {
    #[inline]
    pub(crate) const fn new(id: u32, name: &'static str) -> Self {
        Self {
            id: Some(id),
            name: UStr::Static(name),
        }
    }

    /// Create a `PgTypeInfo` from a type name.
    ///
    /// The OID for the type will be fetched from Postgres on bind or decode of
    /// a value of this type. The fetched OID will be cached per-connection.
    #[inline]
    pub const fn with_name(name: &'static str) -> Self {
        Self {
            id: None,
            name: UStr::Static(name),
        }
    }

    // TODO: __type_feature_gate
}

impl Display for PgTypeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if &*self.name == "char" {
            // format char as "char" to match syntax
            f.write_str("\"char\"")
        } else if &*self.name == "_char" {
            // format _char as "char"[] to match syntax
            f.write_str("\"char\"[]")
        } else if &*self.name == "bpchar" {
            // format bpchar as char to match syntax
            f.write_str("char")
        } else if &*self.name == "_bpchar" {
            // format _bpchar as char[] to match syntax
            f.write_str("char[]")
        } else if self.name.starts_with('_') {
            // format arrays as T[] over _T to match syntax
            write!(f, "{}[]", &self.name[1..])
        } else {
            // otherwise, just write the name
            f.write_str(&*self.name)
        }
    }
}

impl TypeInfo for PgTypeInfo {}

impl PartialEq<PgTypeInfo> for PgTypeInfo {
    fn eq(&self, other: &PgTypeInfo) -> bool {
        if let (Some(id), Some(other_id)) = (self.id, other.id) {
            // when both types have IDs, this is the fastest method of comparison
            return id == other_id;
        }

        // otherwise, a case-insensitive name comparision
        return self.name.eq_ignore_ascii_case(&*other.name);
    }
}

// NOTE: We could definitely figure out a proc macro to query postgres for a list of
//       built-in OIDs and generate all this.

impl PgTypeInfo {
    /// Try to produce a static PgTypeInfo from a built-in type.
    #[inline]
    pub(crate) fn try_from_id(id: u32) -> Option<Self> {
        static MAP: phf::Map<u32, &'static str> = phf::phf_map! {
            16_u32 => "bool",
            17_u32 => "bytea",
            18_u32 => "char",
            19_u32 => "name",
            20_u32 => "int8",
            21_u32 => "int2",
            23_u32 => "int4",
            25_u32 => "text",
            26_u32 => "oid",
            114_u32 => "json",
            199_u32 => "_json",
            600_u32 => "point",
            601_u32 => "lseg",
            602_u32 => "path",
            603_u32 => "box",
            604_u32 => "polygon",
            628_u32 => "line",
            629_u32 => "_line",
            650_u32 => "cidr",
            651_u32 => "_cidr",
            700_u32 => "float4",
            701_u32 => "float8",
            718_u32 => "circle",
            719_u32 => "_circle",
            774_u32 => "macaddr8",
            775_u32 => "_macaddr8",
            829_u32 => "macaddr",
            869_u32 => "inet",
            1000_u32 => "_bool",
            1001_u32 => "_bytea",
            1002_u32 => "_char",
            1003_u32 => "_name",
            1005_u32 => "_int2",
            1007_u32 => "_int4",
            1009_u32 => "_text",
            1014_u32 => "_char",
            1015_u32 => "_varchar",
            1016_u32 => "_int8",
            1017_u32 => "_point",
            1018_u32 => "_lseg",
            1019_u32 => "_path",
            1020_u32 => "_box",
            1021_u32 => "_float4",
            1022_u32 => "_float8",
            1027_u32 => "_polygon",
            1028_u32 => "_oid",
            1040_u32 => "_macaddr",
            1041_u32 => "_inet",
            1042_u32 => "char",
            1043_u32 => "varchar",
            1082_u32 => "date",
            1083_u32 => "time",
            1114_u32 => "timestamp",
            1115_u32 => "_timestamp",
            1182_u32 => "_date",
            1183_u32 => "_time",
            1184_u32 => "timestamptz",
            1185_u32 => "_timestamptz",
            1231_u32 => "_numeric",
            1266_u32 => "timetz",
            1270_u32 => "_timetz",
            1560_u32 => "bit",
            1561_u32 => "_bit",
            1562_u32 => "varbit",
            1563_u32 => "_varbit",
            1700_u32 => "numeric",
            2249_u32 => "record",
            2281_u32 => "interval",
            2287_u32 => "_record",
            2950_u32 => "uuid",
            2951_u32 => "_uuid",
            3802_u32 => "jsonb",
            3807_u32 => "_jsonb",
            3904_u32 => "int4range",
            3905_u32 => "_int4range",
            3906_u32 => "numrange",
            3907_u32 => "_numrange",
            3908_u32 => "tsrange",
            3909_u32 => "_tsrange",
            3910_u32 => "tstzrange",
            3911_u32 => "_tstzrange",
            3912_u32 => "daterange",
            3913_u32 => "_daterange",
            3926_u32 => "int8range",
            3927_u32 => "_int8range",
            4072_u32 => "jsonpath",
            4073_u32 => "_jsonpath",
        };

        Some(Self::new(id, MAP.get(&id)?))
    }
}

// DEVELOPER PRO TIP: find builtin type OIDs easily by grepping this file
// https://github.com/postgres/postgres/blob/master/src/include/catalog/pg_type.dat
//
// If you have Postgres running locally you can also try
// SELECT oid, typarray FROM pg_type where typname = '<type name>'

impl PgTypeInfo {
    // boolean, state of true or false
    pub(crate) const BOOL: Self = Self::new(16, "bool");
    pub(crate) const BOOL_ARRAY: Self = Self::new(1000, "_bool");

    // binary data types, variable-length binary string
    pub(crate) const BYTEA: Self = Self::new(17, "bytea");
    pub(crate) const BYTEA_ARRAY: Self = Self::new(1001, "_bytea");

    // uuid
    pub(crate) const UUID: Self = Self::new(2950, "uuid");
    pub(crate) const UUID_ARRAY: Self = Self::new(2951, "_uuid");

    // record
    pub(crate) const RECORD: Self = Self::new(2249, "record");
    pub(crate) const RECORD_ARRAY: Self = Self::new(2287, "_record");

    //
    // JSON types
    // https://www.postgresql.org/docs/current/datatype-json.html
    //

    pub(crate) const JSON: Self = Self::new(114, "json");
    pub(crate) const JSON_ARRAY: Self = Self::new(199, "_json");

    pub(crate) const JSONB: Self = Self::new(3802, "jsonb");
    pub(crate) const JSONB_ARRAY: Self = Self::new(3807, "_jsonb");

    pub(crate) const JSONPATH: Self = Self::new(4072, "jsonpath");
    pub(crate) const JSONPATH_ARRAY: Self = Self::new(4073, "_jsonpath");

    //
    // network address types
    // https://www.postgresql.org/docs/current/datatype-net-types.html
    //

    pub(crate) const CIDR: Self = Self::new(650, "cidr");
    pub(crate) const CIDR_ARRAY: Self = Self::new(651, "_cidr");

    pub(crate) const INET: Self = Self::new(869, "inet");
    pub(crate) const INET_ARRAY: Self = Self::new(1041, "_inet");

    pub(crate) const MACADDR: Self = Self::new(829, "macaddr");
    pub(crate) const MACADDR_ARRAY: Self = Self::new(1040, "_macaddr");

    pub(crate) const MACADDR8: Self = Self::new(774, "macaddr8");
    pub(crate) const MACADDR8_ARRAY: Self = Self::new(775, "_macaddr8");

    //
    // character types
    // https://www.postgresql.org/docs/current/datatype-character.html
    //

    // internal type for object names
    pub(crate) const NAME: Self = Self::new(19, "name");
    pub(crate) const NAME_ARRAY: Self = Self::new(1003, "_name");

    // character type, fixed-length, blank-padded
    pub(crate) const BPCHAR: Self = Self::new(1042, "char");
    pub(crate) const BPCHAR_ARRAY: Self = Self::new(1014, "_char");

    // character type, variable-length with limit
    pub(crate) const VARCHAR: Self = Self::new(1043, "varchar");
    pub(crate) const VARCHAR_ARRAY: Self = Self::new(1015, "_varchar");

    // character type, variable-length
    pub(crate) const TEXT: Self = Self::new(25, "text");
    pub(crate) const TEXT_ARRAY: Self = Self::new(1009, "_text");

    //
    // numeric types
    // https://www.postgresql.org/docs/current/datatype-numeric.html
    //

    // single-byte internal type
    pub(crate) const CHAR: Self = Self::new(18, "char");
    pub(crate) const CHAR_ARRAY: Self = Self::new(1002, "_char");

    // internal type for type ids
    pub(crate) const OID: Self = Self::new(26, "oid");
    pub(crate) const OID_ARRAY: Self = Self::new(1028, "_oid");

    // small-range integer; -32768 to +32767
    pub(crate) const INT2: Self = Self::new(21, "int2");
    pub(crate) const INT2_ARRAY: Self = Self::new(1005, "_int2");

    // typical choice for integer; -2147483648 to +2147483647
    pub(crate) const INT4: Self = Self::new(23, "int4");
    pub(crate) const INT4_ARRAY: Self = Self::new(1007, "_int4");

    // large-range integer; -9223372036854775808 to +9223372036854775807
    pub(crate) const INT8: Self = Self::new(20, "int8");
    pub(crate) const INT8_ARRAY: Self = Self::new(1016, "_int8");

    // variable-precision, inexact, 6 decimal digits precision
    pub(crate) const FLOAT4: Self = Self::new(700, "float4");
    pub(crate) const FLOAT4_ARRAY: Self = Self::new(1021, "_float4");

    // variable-precision, inexact, 15 decimal digits precision
    pub(crate) const FLOAT8: Self = Self::new(701, "float8");
    pub(crate) const FLOAT8_ARRAY: Self = Self::new(1022, "_float8");

    // user-specified precision, exact
    pub(crate) const NUMERIC: Self = Self::new(1700, "numeric");
    pub(crate) const NUMERIC_ARRAY: Self = Self::new(1231, "_numeric");

    //
    // date/time types
    // https://www.postgresql.org/docs/current/datatype-datetime.html
    //

    // both date and time (no time zone)
    pub(crate) const TIMESTAMP: Self = Self::new(1114, "timestamp");
    pub(crate) const TIMESTAMP_ARRAY: Self = Self::new(1115, "_timestamp");

    // both date and time (with time zone)
    pub(crate) const TIMESTAMPTZ: Self = Self::new(1184, "timestamptz");
    pub(crate) const TIMESTAMPTZ_ARRAY: Self = Self::new(1185, "_timestamptz");

    // date (no time of day)
    pub(crate) const DATE: Self = Self::new(1082, "date");
    pub(crate) const DATE_ARRAY: Self = Self::new(1182, "_date");

    // time of day (no date)
    pub(crate) const TIME: Self = Self::new(1083, "time");
    pub(crate) const TIME_ARRAY: Self = Self::new(1183, "_time");

    // time of day (no date), with time zone
    pub(crate) const TIMETZ: Self = Self::new(1266, "timetz");
    pub(crate) const TIMETZ_ARRAY: Self = Self::new(1270, "_timetz");

    // time interval
    pub(crate) const INTERVAL: Self = Self::new(2281, "interval");

    //
    // geometric types
    // https://www.postgresql.org/docs/current/datatype-geometric.html
    //

    // point on a plane
    pub(crate) const POINT: Self = Self::new(600, "point");
    pub(crate) const POINT_ARRAY: Self = Self::new(1017, "_point");

    // infinite line
    pub(crate) const LINE: Self = Self::new(628, "line");
    pub(crate) const LINE_ARRAY: Self = Self::new(629, "_line");

    // finite line segment
    pub(crate) const LSEG: Self = Self::new(601, "lseg");
    pub(crate) const LSEG_ARRAY: Self = Self::new(1018, "_lseg");

    // rectangular box
    pub(crate) const BOX: Self = Self::new(603, "box");
    pub(crate) const BOX_ARRAY: Self = Self::new(1020, "_box");

    // open or closed path
    pub(crate) const PATH: Self = Self::new(602, "path");
    pub(crate) const PATH_ARRAY: Self = Self::new(1019, "_path");

    // polygon
    pub(crate) const POLYGON: Self = Self::new(604, "polygon");
    pub(crate) const POLYGON_ARRAY: Self = Self::new(1027, "_polygon");

    // circle
    pub(crate) const CIRCLE: Self = Self::new(718, "circle");
    pub(crate) const CIRCLE_ARRAY: Self = Self::new(719, "_circle");

    //
    // bit string types
    // https://www.postgresql.org/docs/current/datatype-bit.html
    //

    pub(crate) const BIT: Self = Self::new(1560, "bit");
    pub(crate) const BIT_ARRAY: Self = Self::new(1561, "_bit");

    pub(crate) const VARBIT: Self = Self::new(1562, "varbit");
    pub(crate) const VARBIT_ARRAY: Self = Self::new(1563, "_varbit");

    //
    // range types
    // https://www.postgresql.org/docs/current/rangetypes.html
    //

    pub(crate) const INT4RANGE: Self = Self::new(3904, "int4range");
    pub(crate) const INT4RANGE_ARRAY: Self = Self::new(3905, "_int4range");

    pub(crate) const NUMRANGE: Self = Self::new(3906, "numrange");
    pub(crate) const NUMRANGE_ARRAY: Self = Self::new(3907, "_numrange");

    pub(crate) const TSRANGE: Self = Self::new(3908, "tsrange");
    pub(crate) const TSRANGE_ARRAY: Self = Self::new(3909, "_tsrange");

    pub(crate) const TSTZRANGE: Self = Self::new(3910, "tstzrange");
    pub(crate) const TSTZRANGE_ARRAY: Self = Self::new(3911, "_tstzrange");

    pub(crate) const DATERANGE: Self = Self::new(3912, "daterange");
    pub(crate) const DATERANGE_ARRAY: Self = Self::new(3913, "_daterange");

    pub(crate) const INT8RANGE: Self = Self::new(3926, "int8range");
    pub(crate) const INT8RANGE_ARRAY: Self = Self::new(3927, "_int8range");
}
