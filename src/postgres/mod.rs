mod backend;
mod connection;
mod error;
mod protocol;
mod query;
mod row;
pub mod types;

pub use self::{
    backend::Postgres, connection::PostgresRawConnection, error::PostgresError,
    query::PostgresQueryParameters, row::PostgresRow,
};

#[cfg(test)]
mod tests {
    use super::Postgres;
    use crate::connection::Connection;
    use futures_util::TryStreamExt;

    const DATABASE_URL: &str = "postgres://postgres@127.0.0.1:5432/";

    #[tokio::test]
    async fn it_connects() {
        let _conn = Connection::<Postgres>::establish(DATABASE_URL)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn it_pings() {
        let mut conn = Connection::<Postgres>::establish(DATABASE_URL)
            .await
            .unwrap();

        conn.ping().await.unwrap();
    }

    #[tokio::test]
    async fn it_fails_on_connect_with_an_unknown_user() {
        let res = Connection::<Postgres>::establish("postgres://not_a_user@127.0.0.1:5432/").await;

        match res {
            Err(crate::Error::Database(err)) => {
                assert_eq!(err.message(), "role \"not_a_user\" does not exist");
            }

            _ => panic!("unexpected result"),
        }
    }

    #[tokio::test]
    async fn it_fails_on_connect_with_an_unknown_database() {
        let res =
            Connection::<Postgres>::establish("postgres://postgres@127.0.0.1:5432/fdggsdfgsdaf")
                .await;

        match res {
            Err(crate::Error::Database(err)) => {
                assert_eq!(err.message(), "database \"fdggsdfgsdaf\" does not exist");
            }

            _ => panic!("unexpected result"),
        }
    }

    #[tokio::test]
    async fn it_fetches_tuples_from_a_system_table() {
        let conn = Connection::<Postgres>::establish(DATABASE_URL)
            .await
            .unwrap();

        let roles: Vec<(String, bool)> = crate::query("SELECT rolname, rolsuper FROM pg_roles")
            .fetch(&conn)
            .try_collect()
            .await
            .unwrap();

        // Sanity check to be sure we did indeed fetch tuples
        assert!(roles.binary_search(&("postgres".to_string(), true)).is_ok());
    }
}
