pub struct ConnectOptions<'a> {
    pub host: &'a str,
    pub port: u16,
    pub user: Option<&'a str>,
    pub database: Option<&'a str>,
    pub password: Option<&'a str>,
}

impl<'a> Default for ConnectOptions<'a> {
    #[inline]
    fn default() -> Self {
        Self { host: "localhost", port: 5432, user: None, database: None, password: None }
    }
}

impl<'a> ConnectOptions<'a> {
    #[inline]
    pub fn new() -> Self { Self::default() }

    #[inline]
    pub fn user(mut self, user: &'a str) -> Self {
        self.user = Some(user);
        self
    }

    #[inline]
    pub fn database(mut self, database: &'a str) -> Self {
        self.database = Some(database);
        self
    }

    #[inline]
    pub fn password(mut self, password: &'a str) -> Self {
        self.password = Some(password);
        self
    }
}
