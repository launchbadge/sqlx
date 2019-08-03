use std::io;

pub trait Encode {
    fn encode(&self, buf: &mut Vec<u8>);
}

pub trait BufMut {
    fn put(&mut self, bytes: &[u8]);

    fn put_byte(&mut self, value: u8);

    fn put_int_16(&mut self, value: i16);

    fn put_int_32(&mut self, value: i32);

    fn put_array_int_16(&mut self, values: &[i16]);

    fn put_array_int_32(&mut self, values: &[i32]);

    fn put_str(&mut self, value: &str);
}

impl BufMut for Vec<u8> {
    #[inline]
    fn put(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }

    #[inline]
    fn put_byte(&mut self, value: u8) {
        self.push(value);
    }

    #[inline]
    fn put_int_16(&mut self, value: i16) {
        self.extend_from_slice(&value.to_be_bytes());
    }

    #[inline]
    fn put_int_32(&mut self, value: i32) {
        self.extend_from_slice(&value.to_be_bytes());
    }

    #[inline]
    fn put_str(&mut self, value: &str) {
        self.extend_from_slice(value.as_bytes());
        self.push(0);
    }

    #[inline]
    fn put_array_int_16(&mut self, values: &[i16]) {
        // FIXME: What happens here when len(values) > i16
        self.put_int_16(values.len() as i16);

        for value in values {
            self.put_int_16(*value);
        }
    }

    #[inline]
    fn put_array_int_32(&mut self, values: &[i32]) {
        // FIXME: What happens here when len(values) > i16
        self.put_int_16(values.len() as i16);

        for value in values {
            self.put_int_32(*value);
        }
    }
}
