use byteorder::{BigEndian, ByteOrder};

pub fn header(buf: &mut Vec<u8>, portal: &str, statement: &str, formats: &[u16]) -> (usize, usize) {
    buf.push(b'B');

    // reserve room for the length
    let len_pos = buf.len();
    buf.extend_from_slice(&[0, 0, 0, 0]);

    buf.extend_from_slice(portal.as_bytes());
    buf.push(b'\0');

    buf.extend_from_slice(statement.as_bytes());
    buf.push(b'\0');

    buf.extend_from_slice(&(formats.len() as i16).to_be_bytes());

    for format in formats {
        buf.extend_from_slice(&format.to_be_bytes());
    }

    // reserve room for the values count
    let value_len_pos = buf.len();
    buf.extend_from_slice(&[0, 0]);

    (len_pos, value_len_pos)
}

pub fn value(buf: &mut Vec<u8>, value: &[u8]) {
    buf.extend_from_slice(&(value.len() as u32).to_be_bytes());
    buf.extend_from_slice(value);
}

pub fn value_null(buf: &mut Vec<u8>) {
    buf.extend_from_slice(&(-1_i32).to_be_bytes());
}

pub fn trailer(buf: &mut Vec<u8>, state: (usize, usize), values: usize, result_formats: &[i16]) {
    buf.extend_from_slice(&(result_formats.len() as i16).to_be_bytes());

    for format in result_formats {
        buf.extend_from_slice(&format.to_be_bytes());
    }

    // Calculate and emplace the total len of the message
    let len = buf.len() - state.0;
    BigEndian::write_u32(&mut buf[(state.0)..], len as u32);

    // Emplace the total num of values
    BigEndian::write_u16(&mut buf[(state.1)..], values as u16);
}

#[cfg(test)]
mod tests {
    const BIND: &[u8] = b"B\0\0\0\x16\0\0\0\0\0\x02\0\0\0\x011\0\0\0\x012\0\0";

    #[test]
    fn it_encodes_bind_for_two() {
        let mut buf = Vec::new();

        let state = super::header(&mut buf, "", "", &[]);
        super::value(&mut buf, b"1");
        super::value(&mut buf, b"2");
        super::trailer(&mut buf, state, 2, &[]);

        assert_eq!(buf, BIND);
    }
}
