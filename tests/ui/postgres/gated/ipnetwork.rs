fn main() {
    let _ = sqlx_oldapi::query!("select '127.0.0.1'::inet");

    let _ = sqlx_oldapi::query!("select '2001:4f8:3:ba::/64'::cidr");

    let _ = sqlx_oldapi::query!("select $1::inet", ());

    let _ = sqlx_oldapi::query!("select $1::cidr", ());
}
