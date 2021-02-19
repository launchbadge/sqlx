use crate::{Arguments, Database};

/// A type that may be executed against a SQL executor.
pub trait Execute<'q, 'a, Db: Database>: Send + Sync {
    /// Returns the SQL to be executed.
    fn sql(&self) -> &str;

    /// Returns the arguments for bind variables in the SQL.
    ///
    /// A value of `None` for arguments is different from an empty list of
    /// arguments. The latter instructs SQLx to prepare the SQL command
    /// (with no arguments) and then execute it. The former
    /// will result in a simple and unprepared SQL command.
    ///
    fn arguments(&self) -> Option<&'_ Arguments<'a, Db>> {
        None
    }

    /// Returns `true` if the SQL statement should be cached for re-use.
    fn persistent(&self) -> bool {
        true
    }
}

impl<'q, Db: Database> Execute<'q, '_, Db> for &'q str {
    fn sql(&self) -> &str {
        self
    }
}

impl<Db: Database> Execute<'_, '_, Db> for String {
    fn sql(&self) -> &str {
        self
    }
}

impl<'q, 'a, Db: Database, E: Execute<'q, 'a, Db>> Execute<'q, 'a, Db> for &'_ E {
    fn sql(&self) -> &str {
        (*self).sql()
    }

    fn arguments(&self) -> Option<&'_ Arguments<'a, Db>> {
        (*self).arguments()
    }
}

impl<'q, 'a, Db: Database> Execute<'q, 'a, Db> for (&'q str, Arguments<'a, Db>) {
    fn sql(&self) -> &str {
        self.0
    }

    fn arguments(&self) -> Option<&'_ Arguments<'a, Db>> {
        Some(&self.1)
    }
}

impl<'q, 'a, Db: Database> Execute<'q, 'a, Db> for (&'q String, Arguments<'a, Db>) {
    fn sql(&self) -> &str {
        self.0
    }

    fn arguments(&self) -> Option<&'_ Arguments<'a, Db>> {
        Some(&self.1)
    }
}
impl<'a, Db: Database> Execute<'_, 'a, Db> for (String, Arguments<'a, Db>) {
    fn sql(&self) -> &str {
        &self.0
    }

    fn arguments(&self) -> Option<&'_ Arguments<'a, Db>> {
        Some(&self.1)
    }
}

impl<'q, 'a, Db: Database> Execute<'q, 'a, Db> for (&'q str, &'a Arguments<'a, Db>) {
    fn sql(&self) -> &str {
        self.0
    }

    fn arguments(&self) -> Option<&'_ Arguments<'a, Db>> {
        Some(&self.1)
    }
}

impl<'q, 'a, Db: Database> Execute<'q, 'a, Db> for (&'q String, &'a Arguments<'a, Db>) {
    fn sql(&self) -> &str {
        self.0
    }

    fn arguments(&self) -> Option<&'_ Arguments<'a, Db>> {
        Some(&self.1)
    }
}
impl<'a, Db: Database> Execute<'_, 'a, Db> for (String, &'a Arguments<'a, Db>) {
    fn sql(&self) -> &str {
        &self.0
    }

    fn arguments(&self) -> Option<&'_ Arguments<'a, Db>> {
        Some(&self.1)
    }
}
