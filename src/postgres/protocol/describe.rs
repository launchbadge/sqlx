/// The Describe message (portal variant) specifies the name of an existing portal
/// (or an empty string for the unnamed portal). The response is a RowDescription message
/// describing the rows that will be returned by executing the portal; or a NoData message
/// if the portal does not contain a query that will return rows; or ErrorResponse if there is no such portal.
pub fn portal(buf: &mut Vec<u8>, name: &str) {
    buf.push(b'D');

    let len = 4 + name.len() + 2;
    buf.extend_from_slice(&(len as i32).to_be_bytes());

    buf.push(b'P');

    buf.extend_from_slice(name.as_bytes());
    buf.push(b'\0');
}

/// The Describe message (statement variant) specifies the name of an existing prepared statement
/// (or an empty string for the unnamed prepared statement). The response is a ParameterDescription
/// message describing the parameters needed by the statement, followed by a RowDescription message
/// describing the rows that will be returned when the statement is eventually executed
/// (or a NoData message if the statement will not return rows). ErrorResponse is issued if
/// there is no such prepared statement. Note that since Bind has not yet been issued,
/// the formats to be used for returned columns are not yet known to the backend; the
/// format code fields in the RowDescription message will be zeroes in this case.
pub fn statement(buf: &mut Vec<u8>, name: &str) {
    buf.push(b'D');

    let len = 4 + name.len() + 2;
    buf.extend_from_slice(&(len as i32).to_be_bytes());

    buf.push(b'S');

    buf.extend_from_slice(name.as_bytes());
    buf.push(b'\0');
}

#[cfg(test)]
mod test {
    #[test]
    fn it_encodes_describe_portal() {
        let mut buf = vec![];
        super::portal(&mut buf, "ABC123");

        assert_eq!(&buf, b"D\x00\x00\x00\x0fPABC123\x00");
    }

    #[test]
    fn it_encodes_describe_statement() {
        let mut buf = vec![];
        super::statement(&mut buf, "95 apples");

        assert_eq!(&buf, b"D\x00\x00\x00\x12S95 apples\x00");
    }
}
