/// Specifies the portal name (empty string denotes the unnamed portal) and a maximum
/// result-row count (zero meaning “fetch all rows”). The result-row count is only meaningful
/// for portals containing commands that return row sets; in other cases the command is
/// always executed to completion, and the row count is ignored.
pub fn execute(buf: &mut Vec<u8>, portal: &str, limit: i32) {
    buf.push(b'E');

    let len = 4 + portal.len() + 1 + 4;
    buf.extend_from_slice(&(len as i32).to_be_bytes());

    // portal
    buf.extend_from_slice(portal.as_bytes());
    buf.push(b'\0');

    // limit
    buf.extend_from_slice(&limit.to_be_bytes());
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_encodes_execute() {
        let mut buf = Vec::new();
        super::execute(&mut buf, "", 0);

        assert_eq!(&*buf, b"E\0\0\0\t\0\0\0\0\0");
    }
}
