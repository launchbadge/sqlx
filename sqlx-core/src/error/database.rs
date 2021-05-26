use std::error::Error as StdError;

/// Representing an error that was returned from the database.
///
/// Provides abstract access to information returned from the database about
/// the error.
///
#[allow(clippy::module_name_repetitions)]
pub trait DatabaseError: 'static + StdError + Send + Sync {
    /// Returns the primary, human-readable error message.
    fn message(&self) -> &str;

    #[doc(hidden)]
    fn as_error(&self) -> &(dyn StdError + Send + Sync + 'static);

    #[doc(hidden)]
    fn as_error_mut(&mut self) -> &mut (dyn StdError + Send + Sync + 'static);

    #[doc(hidden)]
    fn into_error(self: Box<Self>) -> Box<dyn StdError + Send + Sync + 'static>;
}

impl dyn DatabaseError {
    /// Downcast a reference to this generic database error to a specific
    /// database error type.
    ///
    /// # Panics
    ///
    /// Panics if the database error type is not `E`. This is a deliberate contrast from
    /// `Error::downcast_ref` which returns `Option<&E>`. In normal usage, you should know the
    /// specific error type. In other cases, use `try_downcast_ref`.
    pub fn downcast_ref<E: DatabaseError>(&self) -> &E {
        self.try_downcast_ref().unwrap_or_else(|| {
            panic!(
                "downcast to wrong DatabaseError type; original error: {}",
                self
            )
        })
    }

    /// Downcast this generic database error to a specific database error type.
    ///
    /// # Panics
    ///
    /// Panics if the database error type is not `E`. This is a deliberate contrast from
    /// `Error::downcast` which returns `Option<E>`. In normal usage, you should know the
    /// specific error type. In other cases, use `try_downcast`.
    pub fn downcast<E: DatabaseError>(self: Box<Self>) -> Box<E> {
        self.try_downcast().unwrap_or_else(|e| {
            panic!(
                "downcast to wrong DatabaseError type; original error: {}",
                e
            )
        })
    }

    /// Downcast a reference to this generic database error to a specific
    /// database error type.
    #[inline]
    pub fn try_downcast_ref<E: DatabaseError>(&self) -> Option<&E> {
        self.as_error().downcast_ref()
    }

    /// Downcast this generic database error to a specific database error type.
    #[inline]
    pub fn try_downcast<E: DatabaseError>(self: Box<Self>) -> std::result::Result<Box<E>, Box<Self>> {
        if self.as_error().is::<E>() {
            Ok(self.into_error().downcast().unwrap())
        } else {
            Err(self)
        }
    }
}