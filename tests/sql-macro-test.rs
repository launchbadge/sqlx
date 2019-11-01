#![feature(proc_macro_hygiene)]

fn main() {
    sqlx_macros::sql!("SELECT * from accounts where id != $1", "");
}
