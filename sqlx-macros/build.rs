fn main() {
    println!("cargo::rustc-check-cfg=cfg(sqlx_macros_namespace)");
    println!("cargo:rerun-if-env-changed=SQLX_NAMESPACE");
}
