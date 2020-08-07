use crate::database::Database;
use crate::to_value::ToValue;
use smallvec::SmallVec;

/// A tuple of SQL arguments to be bound against a query.
///
/// Often when constructing dynamic SQL queries, it can be useful to collect
/// a heterogeneous list of values as the SQL query is built, to later be
/// used to execute the query. As there is no built-in, dynamic heterogeneous
/// list type in Rust, SQLx provides `Arguments` for this purpose.
///
/// ```rust,no_run
/// # use sqlx_core2::arguments::Arguments;
/// # mod sqlx { use sqlx_core2::query::query_with; }
/// # let _ = sqlx_rt::spawn(async move {
/// let mut args = Arguments::with_capacity(12);
/// args.bind(&10);
/// args.bind("Hello, World!");
/// args.bind(&5.12);
///
/// query_with("INSERT INTO table ( a, b, c ) VALUES ( ?, ?, ? )", &args)
///     .execute(&mut conn).await?;
/// #
/// # Ok(())
/// # });
/// ```
pub struct Arguments<'q, DB: Database>(SmallVec<[Argument<'q, DB>; 6]>);

impl<'q, DB: Database> Default for Arguments<'q, DB> {
    fn default() -> Self {
        Arguments::<'q, DB>::new()
    }
}

impl<'q, DB: Database> Arguments<'q, DB> {
    pub fn new() -> Self {
        Arguments(SmallVec::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Arguments(SmallVec::with_capacity(capacity))
    }

    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn bind<T: ToValue<DB>>(&mut self, value: &'q T) {
        self.0.push(Argument { value, erased: false, checked: true });
    }

    // erased: do not send the type information of the parameter
    pub fn bind_erased<T: ToValue<DB>>(&mut self, value: &'q T) {
        self.0.push(Argument { value, erased: true, checked: true });
    }

    // unchecked: do not send the type information of the parameter *and*
    //            do not fail at runtime if the SQL parameter type is
    //            mismatched with the Rust type
    pub fn bind_unchecked<T: ToValue<DB>>(&mut self, value: &'q T) {
        self.0.push(Argument { value, erased: true, checked: false });
    }
}

pub(crate) struct Argument<'q, DB: Database> {
    pub(crate) value: &'q dyn ToValue<DB>,
    pub(crate) erased: bool,
    pub(crate) checked: bool,
}
