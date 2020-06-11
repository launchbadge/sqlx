use std::fmt::Write;
use std::mem;
use std::sync::Arc;

use futures_util::{stream, StreamExt, TryStreamExt};
use hashbrown::HashMap;

use crate::arguments::Arguments;
use crate::describe::Column;
use crate::error::Error;
use crate::ext::ustr::UStr;
use crate::postgres::message::{ParameterDescription, RowDescription};
use crate::postgres::row::PgColumn;
use crate::postgres::type_info::{PgCustomType, PgType, PgTypeKind};
use crate::postgres::{PgArguments, PgConnection, PgTypeInfo, Postgres};
use crate::query_as::{query_as, query_as_with};
use crate::query_scalar::query_scalar;
use futures_core::future::BoxFuture;

impl PgConnection {
    pub(super) async fn handle_row_description(
        &mut self,
        desc: Option<RowDescription>,
        should_fetch: bool,
        // columns: &mut Vec<PgColumn>,
        // column_names: &mut HashMap<UStr, usize>,
    ) -> Result<(), Error> {
        let mut columns = Vec::new();
        let mut column_names = HashMap::new();

        mem::swap(Arc::make_mut(&mut self.scratch_row_columns), &mut columns);
        mem::swap(
            Arc::make_mut(&mut self.scratch_row_column_names),
            &mut column_names,
        );

        columns.clear();
        column_names.clear();

        let desc = if let Some(desc) = desc {
            desc
        } else {
            // no rows
            return Ok(());
        };

        columns.reserve(desc.fields.len());
        column_names.reserve(desc.fields.len());

        for (index, field) in desc.fields.into_iter().enumerate() {
            let name = UStr::from(field.name);

            let type_info = self
                .maybe_fetch_type_info_by_oid(field.data_type_id, should_fetch)
                .await?;

            let column = PgColumn {
                name: name.clone(),
                type_info,
                relation_id: field.relation_id,
                relation_attribute_no: field.relation_attribute_no,
            };

            columns.push(column);
            column_names.insert(name, index);
        }

        mem::swap(Arc::make_mut(&mut self.scratch_row_columns), &mut columns);
        mem::swap(
            Arc::make_mut(&mut self.scratch_row_column_names),
            &mut column_names,
        );

        Ok(())
    }

    pub(super) async fn handle_parameter_description(
        &mut self,
        desc: ParameterDescription,
    ) -> Result<Vec<Option<PgTypeInfo>>, Error> {
        let mut params = Vec::with_capacity(desc.types.len());

        for ty in desc.types {
            params.push(Some(self.maybe_fetch_type_info_by_oid(ty, true).await?));
        }

        Ok(params)
    }

    async fn maybe_fetch_type_info_by_oid(
        &mut self,
        oid: u32,
        should_fetch: bool,
    ) -> Result<PgTypeInfo, Error> {
        // first we check if this is a built-in type
        // in the average application, the vast majority of checks should flow through this
        if let Some(info) = PgTypeInfo::try_from_oid(oid) {
            return Ok(info);
        }

        // next we check a local cache for user-defined type names <-> object id
        if let Some(info) = self.cache_type_info.get(&oid) {
            return Ok(info.clone());
        }

        // fallback to asking the database directly for a type name
        if should_fetch {
            let info = self.fetch_type_by_oid(oid).await?;

            // cache the type name <-> oid relationship in a paired hashmap
            // so we don't come down this road again
            self.cache_type_info.insert(oid, info.clone());
            self.cache_type_oid
                .insert(info.0.name().to_string().into(), oid);

            Ok(info)
        } else {
            // we are not in a place that *can* run a query
            // this generally means we are in the middle of another query
            // this _should_ only happen for complex types sent through the TEXT protocol
            // we're open to ideas to correct this.. but it'd probably be more efficient to figure
            // out a way to "prime" the type cache for connections rather than make this
            // fallback work correctly for complex user-defined types for the TEXT protocol
            Ok(PgTypeInfo(PgType::DeclareWithOid(oid)))
        }
    }

    async fn fetch_type_by_oid(&mut self, oid: u32) -> Result<PgTypeInfo, Error> {
        let (name, category, relation_id): (String, i8, u32) = query_as(
            "SELECT typname, typcategory, typrelid FROM pg_catalog.pg_type WHERE oid = $1",
        )
        .bind(oid)
        .fetch_one(&mut *self)
        .await?;

        match category as u8 {
            b'A' => Err(err_protocol!("user-defined array types are unsupported")),

            b'P' => Err(err_protocol!("pseudo types are unsupported")),

            b'R' => self.fetch_range_by_oid(oid, name).await,

            b'E' => self.fetch_enum_by_oid(oid, name).await,

            b'C' => self.fetch_composite_by_oid(oid, relation_id, name).await,

            _ => Ok(PgTypeInfo(PgType::Custom(Arc::new(PgCustomType {
                kind: PgTypeKind::Simple,
                name: name.into(),
                oid,
            })))),
        }
    }

    async fn fetch_enum_by_oid(&mut self, oid: u32, name: String) -> Result<PgTypeInfo, Error> {
        let variants: Vec<String> = query_scalar(
            r#"
SELECT enumlabel
FROM pg_catalog.pg_enum
WHERE enumtypid = $1
ORDER BY enumsortorder
            "#,
        )
        .bind(oid)
        .fetch_all(self)
        .await?;

        Ok(PgTypeInfo(PgType::Custom(Arc::new(PgCustomType {
            oid,
            name: name.into(),
            kind: PgTypeKind::Enum(Arc::from(variants)),
        }))))
    }

    fn fetch_composite_by_oid(
        &mut self,
        oid: u32,
        relation_id: u32,
        name: String,
    ) -> BoxFuture<'_, Result<PgTypeInfo, Error>> {
        Box::pin(async move {
            let raw_fields: Vec<(String, u32)> = query_as(
                r#"
SELECT attname, atttypid
FROM pg_catalog.pg_attribute
WHERE attrelid = $1
AND NOT attisdropped
AND attnum > 0
ORDER BY attnum
                "#,
            )
            .bind(relation_id)
            .fetch_all(&mut *self)
            .await?;

            let mut fields = Vec::new();

            for (field_name, field_oid) in raw_fields.into_iter() {
                let field_type = self.maybe_fetch_type_info_by_oid(field_oid, true).await?;

                fields.push((field_name, field_type));
            }

            Ok(PgTypeInfo(PgType::Custom(Arc::new(PgCustomType {
                oid,
                name: name.into(),
                kind: PgTypeKind::Composite(Arc::from(fields)),
            }))))
        })
    }

    async fn fetch_range_by_oid(&mut self, oid: u32, name: String) -> Result<PgTypeInfo, Error> {
        let _: i32 = query_scalar(
            r#"
SELECT 1
FROM pg_catalog.pg_range
WHERE rngtypid = $1
            "#,
        )
        .bind(oid)
        .fetch_one(self)
        .await?;

        let pg_type = PgType::try_from_oid(oid).ok_or_else(|| {
            err_protocol!("Trying to retrieve a DB type that doesn't exist in SQLx")
        })?;

        Ok(PgTypeInfo(PgType::Custom(Arc::new(PgCustomType {
            kind: PgTypeKind::Range(PgTypeInfo(pg_type)),
            name: name.into(),
            oid,
        }))))
    }

    pub(crate) async fn fetch_type_id_by_name(&mut self, name: &str) -> Result<u32, Error> {
        if let Some(oid) = self.cache_type_oid.get(name) {
            return Ok(*oid);
        }

        // language=SQL
        let (oid,): (u32,) = query_as(
            "
SELECT oid FROM pg_catalog.pg_type WHERE typname ILIKE $1
                ",
        )
        .bind(name)
        .fetch_one(&mut *self)
        .await?;

        self.cache_type_oid.insert(name.to_string().into(), oid);

        Ok(oid)
    }

    pub(crate) async fn map_result_columns(
        &mut self,
        columns: &[PgColumn],
    ) -> Result<Vec<Column<Postgres>>, Error> {
        if columns.is_empty() {
            return Ok(vec![]);
        }

        let mut query = String::from("SELECT col.idx, pg_attribute.attnotnull FROM (VALUES ");
        let mut args = PgArguments::default();

        for (i, (column, bind)) in columns.iter().zip((1..).step_by(3)).enumerate() {
            if !args.buffer.is_empty() {
                query += ", ";
            }

            let _ = write!(
                query,
                "(${}::int4, ${}::int4, ${}::int2)",
                bind,
                bind + 1,
                bind + 2
            );

            args.add(i as i32);
            args.add(column.relation_id);
            args.add(column.relation_attribute_no);
        }

        query.push_str(
            ") as col(idx, table_id, col_idx) \
            LEFT JOIN pg_catalog.pg_attribute \
                ON table_id IS NOT NULL \
               AND attrelid = table_id \
               AND attnum = col_idx \
            ORDER BY col.idx",
        );

        query_as_with::<_, (i32, Option<bool>), _>(&query, args)
            .fetch(self)
            .zip(stream::iter(columns.iter().enumerate()))
            .map(|(row, (field_idx, column))| -> Result<Column<_>, Error> {
                let (idx, not_null) = row?;

                // NOTE: it should be impossible for this to fire
                debug_assert_eq!(idx, field_idx as i32);

                Ok(Column {
                    name: column.name.to_string(),
                    type_info: Some(column.type_info.clone()),
                    not_null,
                })
            })
            .try_collect()
            .await
    }
}
