pub trait WriteExt {
    fn write_str_nul(&mut self, s: &str);
    fn write_maybe_str_nul(&mut self, s: Option<&str>);
}

impl WriteExt for Vec<u8> {
    fn write_str_nul(&mut self, s: &str) {
        self.reserve(s.len() + 1);
        self.extend_from_slice(s.as_bytes());
        self.push(0);
    }

    fn write_maybe_str_nul(&mut self, s: Option<&str>) {
        if let Some(s) = s {
            self.reserve(s.len() + 1);
            self.extend_from_slice(s.as_bytes());
        }

        self.push(0);
    }
}

#[cfg(test)]
mod tests {
    use super::WriteExt;

    #[test]
    fn write_str() {
        let mut buf = Vec::new();
        buf.write_str_nul("this is a random dice roll");

        assert_eq!(&buf, b"this is a random dice roll\0");
    }
}
