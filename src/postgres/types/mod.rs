mod boolean;
mod character;
mod numeric;

pub use self::boolean::Bool;

pub struct TypeMetadata {
    pub oid: u32,
    pub array_oid: u32,
}
