use crate::database::HasOutput;
use crate::{encode, Database, TypeEncode};

/// A collection of arguments to be applied to a prepared statement.
///
/// This container allows for a heterogeneous list of positional and named
/// arguments to be collected before executing the query.
///
/// The [`Query`] object uses an internal `Arguments` collection.
///
pub struct Arguments<'a, Db: Database> {
    named: Vec<(&'a str, Argument<'a, Db>)>,
    positional: Vec<Argument<'a, Db>>,
}

/// A single argument to be applied to a prepared statement.
pub struct Argument<'a, Db: Database> {
    unchecked: bool,

    // TODO: we might want to allow binding to Box<dyn TypeEncode<Db>>
    //       this would allow an Owned storage of values
    value: &'a dyn TypeEncode<Db>,
}

impl<Db: Database> Default for Arguments<'_, Db> {
    fn default() -> Self {
        Self { named: Vec::new(), positional: Vec::new() }
    }
}

impl<'a, Db: Database> Arguments<'a, Db> {
    /// Creates an empty `Arguments`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a value to the end of the arguments collection.
    ///
    /// When the argument is applied to a prepared statement, its type will be checked
    /// for compatibility against the expected type from the database. As an example, given a
    /// SQL expression such as `SELECT * FROM table WHERE field = {}`, if `field` is an integer type
    /// and you attempt to bind a `&str` in Rust, an incompatible type error will be raised.
    ///
    pub fn add<T: 'a + TypeEncode<Db>>(&mut self, value: &'a T) {
        self.positional.push(Argument { value, unchecked: false });
    }

    /// Add an unchecked value to the end of the arguments collection.
    ///
    /// When the argument is applied to a prepared statement, its type will not be checked
    /// against the expected type from the database. Further, in PostgreSQL, the argument type
    /// will not be hinted when preparing the statement.
    ///
    pub fn add_unchecked<T: 'a + TypeEncode<Db>>(&mut self, value: &'a T) {
        self.positional.push(Argument { value, unchecked: true });
    }

    /// Add a named value to the argument collection.
    pub fn add_as<T: 'a + TypeEncode<Db>>(&mut self, name: &'a str, value: &'a T) {
        self.named.push((name, Argument { value, unchecked: false }));
    }

    /// Add an unchecked, named value to the arguments collection.
    pub fn add_unchecked_as<T: 'a + TypeEncode<Db>>(&mut self, name: &'a str, value: &'a T) {
        self.named.push((name, Argument { value, unchecked: true }));
    }
}

impl<'a, Db: Database> Arguments<'a, Db> {
    /// Reserves capacity for at least `additional` more positional parameters.
    pub fn reserve_positional(&mut self, additional: usize) {
        self.positional.reserve(additional);
    }

    /// Reserves capacity for at least `additional` more named parameters.
    pub fn reserve_named(&mut self, additional: usize) {
        self.named.reserve(additional);
    }

    /// Returns the number of positional and named parameters.
    #[must_use]
    pub fn len(&self) -> usize {
        self.num_named() + self.num_positional()
    }

    /// Returns `true` if there are no positional or named parameters.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clears the `Arguments`, removing all values.
    pub fn clear(&mut self) {
        self.named.clear();
        self.positional.clear();
    }

    /// Returns the number of named parameters.
    #[must_use]
    pub fn num_named(&self) -> usize {
        self.named.len()
    }

    /// Returns the number of positional parameters.
    #[must_use]
    pub fn num_positional(&self) -> usize {
        self.positional.len()
    }

    /// Returns a reference to the argument at the location, if present.
    pub fn get<'x, I: ArgumentIndex<'a, Db>>(&'x self, index: &I) -> Option<&'x Argument<'a, Db>> {
        index.get(self)
    }
}

impl<'a, Db: Database> Argument<'a, Db> {
    /// Returns `true` if the argument is unchecked.
    #[must_use]
    pub fn unchecked(&self) -> bool {
        self.unchecked
    }

    /// Returns the SQL type identifier of the argument.
    ///
    /// When the statement is prepared, the database will often infer the type
    /// of the incoming argument. This method takes that (`ty`) along with the value of
    /// the argument to determine the actual type identifier that will be sent when
    /// the statement is executed.
    ///
    #[must_use]
    pub fn type_id(&self, ty: &Db::TypeInfo) -> Db::TypeId {
        self.value.type_id(ty)
    }

    /// Encode this argument into the output buffer, for use in executing the prepared statement.
    ///
    /// When the statement is prepared, the database will often infer the type
    /// of the incoming argument. This method takes  that (`ty`) along with the value of
    /// the argument to encode into the output buffer.
    ///
    pub fn encode<'x>(
        &self,
        ty: &Db::TypeInfo,
        out: &mut <Db as HasOutput<'x>>::Output,
    ) -> encode::Result<()> {
        self.value.encode(ty, out)
    }
}

/// A helper trait used for indexing into an [`Arguments`] collection.
pub trait ArgumentIndex<'a, Db: Database> {
    /// Returns a reference to the argument at this location, if present.
    fn get<'x>(&self, arguments: &'x Arguments<'a, Db>) -> Option<&'x Argument<'a, Db>>;
}

// access a named argument by name
impl<'a, Db: Database> ArgumentIndex<'a, Db> for str {
    fn get<'x>(&self, arguments: &'x Arguments<'a, Db>) -> Option<&'x Argument<'a, Db>> {
        arguments.named.iter().find_map(|(name, arg)| (*name == self).then(|| arg))
    }
}

// access a positional argument by index
impl<'a, Db: Database> ArgumentIndex<'a, Db> for usize {
    fn get<'x>(&self, arguments: &'x Arguments<'a, Db>) -> Option<&'x Argument<'a, Db>> {
        arguments.positional.get(*self)
    }
}
