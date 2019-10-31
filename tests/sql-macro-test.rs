fn main() {
    sqlx::sql!("SELECT * from accounts");
}
