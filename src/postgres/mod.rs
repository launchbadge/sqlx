mod backend;
mod connection;
mod error;
mod protocol;
mod query;
mod raw;
mod row;
pub mod types;

pub use self::{
    backend::Postgres, error::PostgresDatabaseError, query::PostgresQueryParameters,
    raw::PostgresRawConnection, row::PostgresRow,
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
        let conn = Connection::<Postgres>::establish(DATABASE_URL)
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
    async fn it_fetches_tuples_from_system_roles() {
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

    #[tokio::test]
    async fn it_fetches_nothing_for_no_rows_from_system_roles() {
        let conn = Connection::<Postgres>::establish(DATABASE_URL)
            .await
            .unwrap();

        let res: Option<(String, bool)> =
            crate::query("SELECT rolname, rolsuper FROM pg_roles WHERE rolname = 'not-a-user'")
                .fetch_optional(&conn)
                .await
                .unwrap();

        assert!(res.is_none());

        let res: crate::Result<(String, bool)> =
            crate::query("SELECT rolname, rolsuper FROM pg_roles WHERE rolname = 'not-a-user'")
                .fetch_one(&conn)
                .await;

        matches::assert_matches!(res, Err(crate::Error::NotFound));
    }

    #[tokio::test]
    async fn it_errors_on_fetching_more_than_one_row_from_system_roles() {
        let conn = Connection::<Postgres>::establish(DATABASE_URL)
            .await
            .unwrap();

        let res: crate::Result<(String, bool)> =
            crate::query("SELECT rolname, rolsuper FROM pg_roles")
                .fetch_one(&conn)
                .await;

        matches::assert_matches!(res, Err(crate::Error::FoundMoreThanOne));
    }

    #[tokio::test]
    async fn it_fetches_one_row_from_system_roles() {
        let conn = Connection::<Postgres>::establish(DATABASE_URL)
            .await
            .unwrap();

        let res: (String, bool) =
            crate::query("SELECT rolname, rolsuper FROM pg_roles WHERE rolname = 'postgres'")
                .fetch_one(&conn)
                .await
                .unwrap();

        assert_eq!(res.0, "postgres");
        assert!(res.1);
    }
}
