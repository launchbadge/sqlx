pub trait Column {
    /// Returns the name or alias of the column.
    fn name(&self) -> &str;

    /// Returns the ordinal (also known as the index) of the column.
    fn ordinal(&self) -> usize;
}
