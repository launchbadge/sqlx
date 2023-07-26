fn main() {
    // we're probably not going to get around to the geometric types anytime soon
    let _ = sqlx_oldapi::query!("select null::circle");
    let _ = sqlx_oldapi::query!("select $1::circle", panic!());
}
