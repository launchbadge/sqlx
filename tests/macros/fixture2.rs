fn main() {}

mod test {
    #[sqlx::test(fixtures(path="../fixtures/postgres", scripts))]
    async fn should_fail(_pool: sqlx::PgPool) {}
}
