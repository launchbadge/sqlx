use std::fmt::Debug;
use std::str::FromStr;

/// Options which can be used to configure how a SQL connection is opened.
///
/// Connection options can be parsed from a connection URI, of the following general format:
///
/// ```text
/// scheme://[username[:password]@][host[:[port]][/database][?options]
/// ```
///
/// The parsing of this URI is database-dependent but the general form should
/// remain similar.
///
#[allow(clippy::module_name_repetitions)]
pub trait ConnectOptions:
    'static + Sized + Send + Sync + Default + Debug + Clone + FromStr<Err = crate::Error>
{
    /// Parse connection options from a connection URI.
    #[inline]
    fn parse(uri: &str) -> crate::Result<Self> {
        uri.parse()
    }

    /// Establish a new connection to the database.
    #[cfg(feature = "async")]
    #[inline]
    fn connect<C, Rt>(&self) -> futures_util::future::BoxFuture<'_, crate::Result<C>>
    where
        C: crate::Connect<Rt, Options = Self> + Sized,
        Rt: crate::Async,
    {
        C::connect_with(self)
    }
}

// FUTURE: Methods which are common across the majority of connection option types could
//         be enshrined into the trait definition.

// FUTURE: Provide a <NetConnectOptions> trait to enshrine common connect option
//         methods across all connection types that connect to a remote database.
