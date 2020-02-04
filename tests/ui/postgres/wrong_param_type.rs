fn main() {
    let _query = sqlx::query!("select $1::text", 0i32);

    let _query = sqlx::query!("select $1::text", Some(0i32));

    let arg = Some(0i32);
    let _query = sqlx::query!("select $1::text", arg);

    let arg = 0i32;
    let _query = sqlx::query!("select $1::text", arg);
}
