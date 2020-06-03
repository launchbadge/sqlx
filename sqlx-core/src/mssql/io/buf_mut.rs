pub trait MsSqlBufMutExt {
    fn put_utf16_str(&mut self, s: &str);
}

impl MsSqlBufMutExt for Vec<u8> {
    fn put_utf16_str(&mut self, s: &str) {
        let mut enc = s.encode_utf16();
        while let Some(ch) = enc.next() {
            self.extend_from_slice(&ch.to_le_bytes());
        }
    }
}
