fn main() {
    let _ = sqlx_oldapi::query!("select CONVERT(now(), DATE) date");

    let _ = sqlx_oldapi::query!("select CONVERT(now(), TIME) time");

    let _ = sqlx_oldapi::query!("select CONVERT(now(), DATETIME) datetime");
}
