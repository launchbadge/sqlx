use crate::aurora::arguments::AuroraArguments;
use crate::aurora::column::AuroraColumn;
use crate::aurora::type_info::AuroraTypeInfo;
use crate::aurora::Aurora;
use crate::column::ColumnIndex;
use crate::error::Error;
use crate::ext::ustr::UStr;
use crate::statement::Statement;
use crate::HashMap;
use either::Either;
use std::borrow::Cow;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AuroraStatement<'q> {
    pub(crate) sql: Cow<'q, str>,
    pub(crate) metadata: Arc<AuroraStatementMetadata>,
}

#[derive(Debug, Default)]
pub(crate) struct AuroraStatementMetadata {
    pub(crate) columns: Vec<AuroraColumn>,
    pub(crate) column_names: HashMap<UStr, usize>,
    pub(crate) parameters: Vec<AuroraTypeInfo>,
}

impl<'q> Statement<'q> for AuroraStatement<'q> {
    type Database = Aurora;

    fn to_owned(&self) -> AuroraStatement<'static> {
        AuroraStatement::<'static> {
            sql: Cow::Owned(self.sql.clone().into_owned()),
            metadata: self.metadata.clone(),
        }
    }

    fn sql(&self) -> &str {
        &self.sql
    }

    fn parameters(&self) -> Option<Either<&[AuroraTypeInfo], usize>> {
        Some(Either::Left(&self.metadata.parameters))
    }

    fn columns(&self) -> &[AuroraColumn] {
        &self.metadata.columns
    }

    impl_statement_query!(AuroraArguments);
}

impl ColumnIndex<AuroraStatement<'_>> for &'_ str {
    fn index(&self, statement: &AuroraStatement<'_>) -> Result<usize, Error> {
        statement
            .metadata
            .column_names
            .get(*self)
            .ok_or_else(|| Error::ColumnNotFound((*self).into()))
            .map(|v| *v)
    }
}

#[cfg(feature = "any")]
impl<'q> From<AuroraStatement<'q>> for crate::any::AnyStatement<'q> {
    #[inline]
    fn from(statement: AuroraStatement<'q>) -> Self {
        crate::any::AnyStatement::<'q> {
            columns: statement
                .metadata
                .columns
                .iter()
                .map(|col| col.clone().into())
                .collect(),
            column_names: std::sync::Arc::new(statement.metadata.column_names.clone()),
            parameters: Some(Either::Left(
                statement
                    .metadata
                    .parameters
                    .iter()
                    .map(|ty| ty.clone().into())
                    .collect(),
            )),
            sql: statement.sql,
        }
    }
}
