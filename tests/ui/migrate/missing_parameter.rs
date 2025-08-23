fn main() {
    //Fails due to missing migration parameter 
    let _shaggy = "shaggy";
    sqlx::migrate!("../../../../tests/ui/migrate/migrations", parameters = [("my_user", "scooby"), ("fooby", _shaggy)]);
}
