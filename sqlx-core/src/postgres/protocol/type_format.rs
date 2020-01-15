#[derive(Debug, Copy, Clone)]
#[repr(i16)]
pub enum TypeFormat {
    Text = 0,
    Binary = 1,
}

impl From<i16> for TypeFormat {
    fn from(code: i16) -> TypeFormat {
        match code {
            0 => TypeFormat::Text,
            1 => TypeFormat::Binary,

            _ => unreachable!(),
        }
    }
}
