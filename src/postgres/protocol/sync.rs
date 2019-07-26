/// This parameterless message causes the backend to close the current transaction if it's not inside
/// a BEGIN/COMMIT transaction block (“close” meaning to commit if no error, or roll back if error).
/// Then a ReadyForQuery response is issued.
pub fn sync(buf: &mut Vec<u8>) {
    buf.push(b'S');
    buf.extend_from_slice(&4_i32.to_be_bytes());
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_encodes_sync() {
        let mut buf = Vec::new();
        super::sync(&mut buf);

        assert_eq!(&*buf, b"S\0\0\0\x04");
    }
}
