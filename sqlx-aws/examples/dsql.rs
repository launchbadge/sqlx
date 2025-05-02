use aws_sdk_dsql::error::BoxError;
use sqlx_aws::iam::dsql::DsqlIamProvider;
use sqlx_postgres::{PgConnectOptions, PgPoolOptions};

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let hostname = std::env::var("DSQL_CLUSTER_ENDPOINT")
        .expect("please set DSQL_CLUSTER_ENDPOINT is your environment");

    let provider = DsqlIamProvider::new(hostname).await?;
    let opts = PgConnectOptions::new_without_pgpass()
        .password(provider)
        .database("postgres");
    let _pool = PgPoolOptions::new().connect_with(opts).await?;

    Ok(())
}
