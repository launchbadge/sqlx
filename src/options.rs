use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq)]
pub struct ConnectOptions<'a> {
    pub host: Cow<'a, str>,
    pub port: u16,
    pub user: Option<Cow<'a, str>>,
    pub database: Option<Cow<'a, str>>,
    pub password: Option<Cow<'a, str>>,
}

impl<'a> Default for ConnectOptions<'a> {
    #[inline]
    fn default() -> Self {
        Self {
            host: Cow::Borrowed("localhost"),
            port: 5432,
            user: None,
            database: None,
            password: None,
        }
    }
}

impl<'a> ConnectOptions<'a> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn into_owned(self) -> ConnectOptions<'static> {
        ConnectOptions {
            host: self.host.into_owned().into(),
            port: self.port,
            user: self.user.map(|s| s.into_owned().into()),
            database: self.database.map(|s| s.into_owned().into()),
            password: self.password.map(|s| s.into_owned().into()),
        }
    }

    #[inline]
    pub fn host(mut self, host: &'a str) -> Self {
        self.host = Cow::Borrowed(host);
        self
    }

    #[inline]
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    #[inline]
    pub fn user(mut self, user: &'a str) -> Self {
        self.user = Some(Cow::Borrowed(user));
        self
    }

    #[inline]
    pub fn database(mut self, database: &'a str) -> Self {
        self.database = Some(Cow::Borrowed(database));
        self
    }

    #[inline]
    pub fn password(mut self, password: &'a str) -> Self {
        self.password = Some(Cow::Borrowed(password));
        self
    }
}
