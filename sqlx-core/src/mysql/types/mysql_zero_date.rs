#[derive(Debug, PartialEq)]
pub enum MysqlZeroDate<T> {
    Zero,
    NotZero(T),
}

#[derive(Debug, Clone)]
struct ZeroDateError;

impl std::fmt::Display for ZeroDateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unexpected ZERO_DATE encountered!")
    }
}

impl std::error::Error for ZeroDateError {}
