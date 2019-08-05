// TODO: Better name for ToSql/ToSqlAs. ToSqlAs is the _conversion_ trait.
//       ToSql is type fallback for Rust/SQL (e.g., what is the probable SQL type for this Rust type)

pub trait SqlType {
    // FIXME: This is a postgres thing
    const OID: u32;
}

pub trait ToSql {
    /// SQL type that should be inferred from the implementing Rust type.
    type Type: SqlType;
}

pub trait ToSqlAs<T: SqlType>: ToSql {
    fn to_sql(self, buf: &mut Vec<u8>);
}
