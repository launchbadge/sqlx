use std::fmt::{Debug, Display};

pub trait TypeInfo: Debug + Display + Clone + PartialEq<Self> {}
