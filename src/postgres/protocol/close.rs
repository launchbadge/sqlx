
pub fn portal(buf: &mut Vec<u8>, name: &str) {
    buf.push(b'C');

    let len = 4 + name.len() + 2;
    buf.extend_from_slice(&(len as i32).to_be_bytes());

    buf.push(b'P');

    buf.extend_from_slice(name.as_bytes());
    buf.push(b'\0');
}

pub fn statement(buf: &mut Vec<u8>, name: &str) {
    buf.push(b'C');

    let len = 4 + name.len() + 2;
    buf.extend_from_slice(&(len as i32).to_be_bytes());

    buf.push(b'S');

    buf.extend_from_slice(name.as_bytes());
    buf.push(b'\0');
}

#[cfg(test)]
mod test {
    #[test]
    fn it_encodes_close_portal() {
        let mut buf = vec![];
        super::portal(&mut buf, "ABC123");

        assert_eq!(&buf, b"C\x00\x00\x00\x0fPABC123\x00");
    }

    #[test]
    fn it_encodes_close_statement() {
        let mut buf = vec![];
        super::statement(&mut buf, "95 apples");

        assert_eq!(&buf, b"C\x00\x00\x00\x12S95 apples\x00");
    }
}
