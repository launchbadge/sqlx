use sqlx::postgres::Postgres;
use sqlx::query_builder::QueryBuilder;
use sqlx::Execute;

#[test]
fn test_new() {
    let qb: QueryBuilder<'_, Postgres> = QueryBuilder::new("SELECT * FROM users");
    assert_eq!(qb.sql(), "SELECT * FROM users");
}

#[test]
fn test_push() {
    let mut qb: QueryBuilder<'_, Postgres> = QueryBuilder::new("SELECT * FROM users");
    let second_line = " WHERE last_name LIKE '[A-N]%';";
    qb.push(second_line);

    assert_eq!(
        qb.sql(),
        "SELECT * FROM users WHERE last_name LIKE '[A-N]%';".to_string(),
    );
}

#[test]
#[should_panic]
fn test_push_panics_after_build_without_reset() {
    let mut qb: QueryBuilder<'_, Postgres> = QueryBuilder::new("SELECT * FROM users;");

    let _query = qb.build();

    qb.push("SELECT * FROM users;");
}

#[test]
fn test_push_bind() {
    let mut qb: QueryBuilder<'_, Postgres> = QueryBuilder::new("SELECT * FROM users WHERE id = ");

    qb.push_bind(42i32)
        .push(" OR membership_level = ")
        .push_bind(3i32);

    assert_eq!(
        qb.sql(),
        "SELECT * FROM users WHERE id = $1 OR membership_level = $2"
    );
}

#[test]
fn test_build() {
    let mut qb: QueryBuilder<'_, Postgres> = QueryBuilder::new("SELECT * FROM users");

    qb.push(" WHERE id = ").push_bind(42i32);
    let query = qb.build();

    assert_eq!(query.sql(), "SELECT * FROM users WHERE id = $1");
    assert_eq!(Execute::persistent(&query), true);
}

#[test]
fn test_reset() {
    let mut qb: QueryBuilder<'_, Postgres> = QueryBuilder::new("");

    let _query = qb
        .push("SELECT * FROM users WHERE id = ")
        .push_bind(42i32)
        .build();

    qb.reset();

    assert_eq!(qb.sql(), "");
}

#[test]
fn test_query_builder_reuse() {
    let mut qb: QueryBuilder<'_, Postgres> = QueryBuilder::new("");

    let _query = qb
        .push("SELECT * FROM users WHERE id = ")
        .push_bind(42i32)
        .build();

    qb.reset();

    let query = qb.push("SELECT * FROM users WHERE id = 99").build();

    assert_eq!(query.sql(), "SELECT * FROM users WHERE id = 99");
}

#[test]
fn test_query_builder_with_args() {
    let mut qb: QueryBuilder<'_, Postgres> = QueryBuilder::new("");

    let mut query = qb
        .push("SELECT * FROM users WHERE id = ")
        .push_bind(42i32)
        .build();

    let mut qb: QueryBuilder<'_, Postgres> =
        QueryBuilder::with_arguments(query.sql(), query.take_arguments().unwrap().unwrap());
    let query = qb.push(" OR membership_level = ").push_bind(3i32).build();

    assert_eq!(
        query.sql(),
        "SELECT * FROM users WHERE id = $1 OR membership_level = $2"
    );
}
