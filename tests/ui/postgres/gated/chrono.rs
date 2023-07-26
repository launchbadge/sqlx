fn main() {
    let _ = sqlx_oldapi::query!("select now()::date");

    let _ = sqlx_oldapi::query!("select now()::time");

    let _ = sqlx_oldapi::query!("select now()::timestamp");

    let _ = sqlx_oldapi::query!("select now()::timestamptz");

    let _ = sqlx_oldapi::query!("select $1::date", ());

    let _ = sqlx_oldapi::query!("select $1::time", ());

    let _ = sqlx_oldapi::query!("select $1::timestamp", ());

    let _ = sqlx_oldapi::query!("select $1::timestamptz", ());
}
