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
