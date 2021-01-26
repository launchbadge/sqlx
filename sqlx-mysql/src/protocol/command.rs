/// To signal the start of the command phase, the MySQL connection needs to reset the
/// packet sequence ID to `0`.
///
/// Each serializable protocol type in sqlx-mysql implements `MaybeCommand` which lets
/// the connection declaratively decide to reset the sequence ID.
///
/// Default implementing `MaybeCommand` will declare the type to *NOT* be a command. The `Command`
/// marker trait is available to make it one-line to override the `is_command` function.
///
pub(crate) trait MaybeCommand {
    fn is_command() -> bool {
        false
    }
}

// raw bytes are not a command
impl MaybeCommand for &'_ [u8] {}

/// Marker trait to signal that this protocol type is a Command.
pub(crate) trait Command: MaybeCommand {}

impl<C> MaybeCommand for C
where
    C: Command,
{
    #[inline]
    fn is_command() -> bool {
        true
    }
}
