use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Default, Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum OrderBy {
    #[default]
    Asc,
    Desc,
}

impl Display for OrderBy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderBy::Asc => write!(f, "ASC"),
            OrderBy::Desc => write!(f, "DESC"),
        }
    }
}
/// `#[derive(ToOrm)]`
/// generate methods for Object Relational Mapping.
///
/// attributes:
///
/// `#[sqlx(pk)]`
/// Set the primary key at creation time as a Uuidv4.
/// Requires the type to be Uuid.
/// Primary key. is automatically considered as a `#[sqlx(by)]` field.
///
/// `#[sqlx(rename="name")]`
/// rename table name or field name.
/// default table name by struct name pluralized to_table_case: UserDetail => user_details.
/// default field name by field name to_snake_case: UserDetail => user_detail.
///
/// `#[sqlx(skip)]`
/// ignore field.
///
/// `#[sqlx(readonly)]`
/// readonly attribute.
/// Can be inserted but not updated.
///
/// `#[sqlx(by)]`
/// generate by_<field>, delete_by_<field> and query with its order_by_<field>, group_by_<field>,
/// limit and offset methods.
///
/// `#[sqlx(created_at)]`
/// Updates the field at creation time.
/// Requires the type to be `chrono::DateTime<FixedOffset>`.
///
/// `#[sqlx(updated_at)]`
/// Updates the field at creation and update time.
/// Requires the type to be `chrono::DateTime<FixedOffset>`.
///
/// `#[sqlx(new="module::path::class::new_custom()")]`
/// Uses a specific function call to create a new instance of the parameter.
/// The function call is expected to return an instance.
/// Defaults to new().
///
/// `#[sqlx(is_default="is_nil()")]`
/// Uses a specific function call to check if the returned value if the default value.
/// The function call is expected to return bool.
/// Defaults to class_type::default() which assumes both the Default trait is implemented.
///

#[derive(Default, Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum WhereOp {
    #[default]
    Equal,
    NotEqual,
    Less,
    LessOrEqual,
    More,
    MoreOrEqual,
}

impl Display for WhereOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WhereOp::Equal => write!(f, "="),
            WhereOp::NotEqual => write!(f, "!="),
            WhereOp::Less => write!(f, "<"),
            WhereOp::LessOrEqual => write!(f, "<="),
            WhereOp::More => write!(f, ">"),
            WhereOp::MoreOrEqual => write!(f, ">="),
        }
    }
}
