fn main() {
    //Fails due to invalid key
    sqlx::migrate!("foo", parameters = [(123, "foo")]);
}
