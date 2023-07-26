fn main() {
    let query = sqlx_oldapi::query!("select 1 as \"'1\"");
}
