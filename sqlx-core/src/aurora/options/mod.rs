use std::env::var;

mod connect;
mod parse;
use crate::connection::LogSettings;

#[derive(Debug, Clone)]
pub struct AuroraConnectOptions {
    pub(crate) region: String,
    pub(crate) resource_arn: Option<String>,
    pub(crate) secret_arn: Option<String>,
    pub(crate) database: Option<String>,
    pub(crate) schema: Option<String>,
    pub(crate) statement_cache_capacity: usize,
    pub(crate) log_settings: LogSettings,
}

impl Default for AuroraConnectOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl AuroraConnectOptions {
    pub fn new() -> Self {
        let region = var("AURORA_REGION")
            .ok()
            .unwrap_or_else(|| "us-east-1".to_owned());

        let resource_arn = var("AURORA_RESOURCE_ARN").ok();
        let secret_arn = var("AURORA_SECRET_ARN").ok();

        AuroraConnectOptions {
            region,
            resource_arn,
            secret_arn,
            database: None,
            schema: None,
            statement_cache_capacity: 100,
            log_settings: Default::default(),
        }
    }

    pub fn region(mut self, region: &str) -> Self {
        self.region = region.to_owned();
        self
    }

    pub fn resource_arn(mut self, resource_arn: &str) -> Self {
        self.resource_arn = Some(resource_arn.to_owned());
        self
    }

    pub fn secret_arn(mut self, secret_arn: &str) -> Self {
        self.secret_arn = Some(secret_arn.to_owned());
        self
    }

    pub fn schema(mut self, schema: &str) -> Self {
        self.schema = Some(schema.to_owned());
        self
    }

    pub fn database(mut self, database: &str) -> Self {
        self.database = Some(database.to_owned());
        self
    }

    /// Sets the capacity of the connection's statement cache in a number of stored
    /// distinct statements. Caching is handled using LRU, meaning when the
    /// amount of queries hits the defined limit, the oldest statement will get
    /// dropped.
    ///
    /// The default cache capacity is 100 statements.
    pub fn statement_cache_capacity(mut self, capacity: usize) -> Self {
        self.statement_cache_capacity = capacity;
        self
    }
}
