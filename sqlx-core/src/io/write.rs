pub trait WriteExt {
    fn write_str_with_nul(&mut self, s: &str);
}

impl WriteExt for Vec<u8> {
    #[inline]
    fn write_str_with_nul(&mut self, s: &str) {
        self.extend(s.as_bytes());
        self.push(0);
    }
}
