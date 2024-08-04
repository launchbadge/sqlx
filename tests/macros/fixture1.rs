fn main() {}

mod test {
    #[sqlx::test(fixtures(path="../fixtures/postgres", scripts("posts")))]
    async fn should_pass(_pool: sqlx::PgPool) {}
}
