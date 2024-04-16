use std::marker::PhantomData;
use std::mem;
use either::Either;
use crate::database::Database;
use crate::from_row::FromRow;
use crate::row::Row;
use crate::sync::spsc;

pub trait ResultSet {
    type Database: Database;

    type Row;

    /// Wait for the server to return the next row in the result set.
    ///
    /// Returns `Ok(None)` when the result set is exhausted. The result of the query
    /// will be available from [`.next_result()`][Self::next_result].
    ///
    /// If the query was capable of returning multiple result sets, this will start returning
    /// rows again after returning `Ok(None)`.
    ///
    /// This clears the stored result if one was available but `.next_result()` was not called.
    async fn next_row(&mut self) -> crate::Result<Option<Self::Row>>;

    /// Wait for the next query result, giving the number of rows affected.
    ///
    /// If [`.next_row()`][Self::next_row] returned `Ok(None)`, the result should be cached
    /// internally and this should return immediately.
    ///
    /// If there are any rows buffered before the next query result, this will discard them.
    async fn next_result(&mut self) -> crate::Result<Option<<Self::Database as Database>::QueryResult>>;

    fn map_row<F, R>(self, map: F) -> MapRow<Self, R, F> where F: FnMut(Self::Row) -> crate::Result<R> {
        MapRow {
            map,
            inner: self,
            row: PhantomData
        }
    }

    fn map_from_row<T>(self) -> MapFromRow<Self, T>
    where
        Self::Row: Row<Database = Self::Database>,
        T: for<'r> FromRow<'r, Self::Row>
    {
        self.map_row(|row| T::from_row(&row))
    }

    async fn collect_rows<T: Default + Extend<Self::Row>>(&mut self) -> crate::Result<T> {
        let mut rows_out = T::default();

        while let Some(row) = self.next_row().await? {
            rows_out.extend(Some(row));
        }

        Ok(rows_out)
    }
}

pub struct MapRow<Rs, Row, F> {
    map: F,
    inner: Rs,
    row: PhantomData<Row>,
}

impl<Rs, Row, F> ResultSet for MapRow<Rs, Row, F>
where
    Rs: ResultSet,
    F: FnMut(Rs::Row) -> Row
{
    type Database = Rs::Database;
    type Row = Row;

    async fn next_row(&mut self) -> crate::Result<Option<Self::Row>> {
        let maybe_row = self.inner.next_row().await?;
        Ok(maybe_row.map(&mut self.map))
    }

    async fn next_result(&mut self) -> crate::Result<Option<<Self::Database as Database>::QueryResult>> {
        self.inner.next_result()
    }
}

pub type MapFromRow<Rs: ResultSet, Row> = MapRow<Rs, Row, fn(Rs::Row) -> Row>;

pub struct ChannelResultSet<DB: Database> {
    flavor: Flavor<DB>,
    last_result: Option<DB::QueryResult>,
}

enum Flavor<DB: Database> {
    Channel(spsc::Receiver<crate::Result<Either<DB::QueryResult, DB::Row>>>),
    Error(crate::Error),
    Empty,
}

impl<DB: Database> ResultSet for ChannelResultSet<DB> {
    type Database = DB;
    type Row = DB::Row;

    async fn next_row(&mut self) -> crate::Result<Option<Self::Row>> {
        // Clear the previous result if it was ignored.
        self.last_result = None;

        match self.flavor.recv().await? {
            Some(Either::Left(result)) => {
                self.last_result = Some(result);
                return Ok(None);
            }
            Some(Either::Right(row)) => Ok(Some(row)),
            None => Ok(None),
        }
    }

    async fn next_result(&mut self) -> crate::Result<Option<<Self::Database as Database>::QueryResult>> {
        if let Some(result) = self.last_result.take() {
            return Ok(Some(result));
        }

        loop {
            match self.flavor.recv().await? {
                Some(Either::Left(result)) => return Ok(Some(result)),
                // Drop rows until the next result.
                Some(Either::Right(_)) => (),
                None => return Ok(None),
            }
        }
    }
}

impl<DB: Database> Flavor<DB> {
    async fn recv(&mut self) -> crate::Result<Option<Either<DB::QueryResult, DB::Row>>> {
        match self {
            Self::Channel(chan) => chan.recv().await.transpose(),
            Self::Error(_) => {
                let Self::Error(e) = mem::replace(self, Self::Empty) else {
                    unreachable!()
                };
                Err(e)
            }
            Self::Empty => Ok(None)
        }
    }
}
