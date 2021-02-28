use std::any;

use crate::database::HasOutput;
use crate::{encode, Database, Error, Result, TypeEncode, TypeInfo};

use std::borrow::Cow;
use std::fmt::{self, Display, Formatter};

/// A collection of arguments to be applied to a prepared statement.
///
/// This container allows for a heterogeneous list of positional and named
/// arguments to be collected before executing the query.
///
/// The [`Query`] object uses an internal `Arguments` collection.
///
pub struct Arguments<'a, Db: Database> {
    named: Vec<Argument<'a, Db>>,
    positional: Vec<Argument<'a, Db>>,
}

/// The index for a given bind argument; either positional, or named.
#[derive(Debug, PartialEq, Eq)]
pub enum ArgumentIndex<'a> {
    Positioned(usize),
    Named(Cow<'a, str>),
}

/// A single argument to be applied to a prepared statement.
pub struct Argument<'a, Db: Database> {
    unchecked: bool,
    index: ArgumentIndex<'a>,

    // preserved from `T::type_id()`
    type_id: Db::TypeId,

    // preserved from `T::compatible`
    type_compatible: fn(&Db::TypeInfo) -> bool,

    // preserved from `any::type_name::<T>`
    // used in error messages
    rust_type_name: &'static str,

    // TODO: we might want to allow binding to Box<dyn TypeEncode<Db>>
    //       this would allow an Owned storage of values
    value: &'a dyn TypeEncode<Db>,
}

impl<'a, Db: Database> Argument<'a, Db> {
    fn new<'b: 'a, T: 'a + TypeEncode<Db>>(
        parameter: impl Into<ArgumentIndex<'b>>,
        value: &'a T,
        unchecked: bool,
    ) -> Self {
        Self {
            value,
            unchecked,
            index: parameter.into(),
            type_id: T::type_id(),
            type_compatible: T::compatible,
            rust_type_name: any::type_name::<T>(),
        }
    }
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
        let index = self.positional.len();

        self.positional.push(Argument::new(index, value, false));
    }

    /// Add an unchecked value to the end of the arguments collection.
    ///
    /// When the argument is applied to a prepared statement, its type will not be checked
    /// against the expected type from the database. Further, in PostgreSQL, the argument type
    /// will not be hinted when preparing the statement.
    ///
    pub fn add_unchecked<T: 'a + TypeEncode<Db>>(&mut self, value: &'a T) {
        let index = self.positional.len();

        self.positional.push(Argument::new(index, value, true));
    }

    /// Add a named value to the argument collection.
    pub fn add_as<T: 'a + TypeEncode<Db>>(&mut self, name: &'a str, value: &'a T) {
        self.named.push(Argument::new(name, value, false));
    }

    /// Add an unchecked, named value to the arguments collection.
    pub fn add_unchecked_as<T: 'a + TypeEncode<Db>>(&mut self, name: &'a str, value: &'a T) {
        self.named.push(Argument::new(name, value, true));
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

    /// Returns an iterator of the positional parameters.
    pub fn positional(&self) -> impl Iterator<Item = &Argument<'a, Db>> {
        self.positional.iter()
    }

    /// Returns an iterator of the named parameters.
    pub fn named(&self) -> impl Iterator<Item = &Argument<'a, Db>> {
        self.named.iter()
    }

    /// Returns a reference to the argument at the location, if present.
    pub fn get<'x, 'i, I: Into<ArgumentIndex<'i>>>(
        &'x self,
        index: I,
    ) -> Option<&'x Argument<'a, Db>> {
        let index = index.into();

        match index {
            ArgumentIndex::Named(_) => &self.named,
            ArgumentIndex::Positioned(_) => &self.positional,
        }
        .iter()
        .find(|arg| arg.index == index)
    }
}

impl<'a, Db: Database> Argument<'a, Db> {
    /// Gets the name of this argument, if it is a named argument, None otherwise
    pub fn name(&self) -> Option<&str> {
        self.index.name()
    }

    /// Gets the position of this argument, if it is a positional argument, None otherwise
    pub fn position(&self) -> Option<usize> {
        self.index.position()
    }

    /// Returns the SQL type identifier of the argument.
    #[must_use]
    pub fn type_id(&self) -> Db::TypeId {
        self.type_id
    }

    /// Encode this argument into the output buffer, for use in executing
    /// the prepared statement.
    ///
    /// When the statement is prepared, the database will often infer the type
    /// of the incoming argument. This method takes  that (`ty`) along with the value of
    /// the argument to encode into the output buffer.
    ///
    pub fn encode<'x>(
        &self,
        ty: &Db::TypeInfo,
        out: &mut <Db as HasOutput<'x>>::Output,
    ) -> Result<encode::IsNull> {
        let res = if !self.unchecked && !ty.is_unknown() && !(self.type_compatible)(ty) {
            Err(encode::Error::TypeNotCompatible {
                rust_type_name: self.rust_type_name,
                sql_type_name: ty.name(),
            })
        } else {
            self.value.encode(ty, out)
        };

        res.map_err(|source| Error::ParameterEncode { parameter: self.index.to_static(), source })
    }

    pub fn value(&self) -> &(dyn TypeEncode<Db> + 'a) {
        self.value
    }
}

impl<'a> From<&'a str> for ArgumentIndex<'a> {
    fn from(name: &'a str) -> Self {
        ArgumentIndex::Named(name.into())
    }
}

impl<'a> From<&'a String> for ArgumentIndex<'a> {
    fn from(name: &'a String) -> Self {
        ArgumentIndex::Named(name.into())
    }
}

impl From<usize> for ArgumentIndex<'static> {
    fn from(position: usize) -> Self {
        ArgumentIndex::Positioned(position)
    }
}

impl<'a, 'b> From<&'a ArgumentIndex<'b>> for ArgumentIndex<'a> {
    fn from(idx: &'a ArgumentIndex<'b>) -> Self {
        match idx {
            ArgumentIndex::Positioned(pos) => ArgumentIndex::Positioned(*pos),
            ArgumentIndex::Named(name) => ArgumentIndex::Named(name.as_ref().into()),
        }
    }
}

impl<'a> ArgumentIndex<'a> {
    pub(crate) fn into_static(self) -> ArgumentIndex<'static> {
        match self {
            Self::Positioned(pos) => ArgumentIndex::Positioned(pos),
            Self::Named(named) => ArgumentIndex::Named(named.into_owned().into()),
        }
    }

    pub(crate) fn to_static(&self) -> ArgumentIndex<'static> {
        match self {
            Self::Positioned(pos) => ArgumentIndex::Positioned(*pos),
            Self::Named(named) => ArgumentIndex::Named((**named).to_owned().into()),
        }
    }

    pub(crate) fn name(&self) -> Option<&str> {
        if let Self::Named(s) = self {
            Some(s)
        } else {
            None
        }
    }

    pub(crate) fn position(&self) -> Option<usize> {
        if let Self::Positioned(pos) = *self {
            Some(pos)
        } else {
            None
        }
    }
}

impl Display for ArgumentIndex<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Positioned(pos) => Display::fmt(pos, f),
            Self::Named(named) => Display::fmt(named, f),
        }
    }
}

impl PartialEq<str> for ArgumentIndex<'_> {
    fn eq(&self, other: &str) -> bool {
        self == &ArgumentIndex::from(other)
    }
}

impl PartialEq<&'_ str> for ArgumentIndex<'_> {
    fn eq(&self, other: &&str) -> bool {
        self == &ArgumentIndex::from(*other)
    }
}

impl PartialEq<usize> for ArgumentIndex<'_> {
    fn eq(&self, other: &usize) -> bool {
        self == &ArgumentIndex::from(*other)
    }
}
