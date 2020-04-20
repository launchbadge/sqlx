pub trait PgWriteExt {
    fn write_length_prefixed<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Vec<u8>);

    fn write_statement_name(&mut self, id: u32);

    fn write_portal_name(&mut self, id: Option<u32>);
}

impl PgWriteExt for Vec<u8> {
    // writes a length-prefixed message, this is used when encoding nearly all messages as postgres
    // wants us to send the length of the often-variable-sized messages up front
    fn write_length_prefixed<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Vec<u8>),
    {
        // reserve space to write the prefixed length
        let offset = self.len();
        self.extend(&[0; 4]);

        // write the main body of the message
        f(self);

        // now calculate the size of what we wrote and set the length value
        let size = (self.len() - offset) as i32;
        self[offset..(offset + 4)].copy_from_slice(&size.to_be_bytes());
    }

    // writes a statement name by ID
    #[inline]
    fn write_statement_name(&mut self, id: u32) {
        self.extend(b"sqlx_s_");

        itoa::write(&mut *self, id).unwrap();

        self.push(0);
    }

    // writes a portal name by ID
    #[inline]
    fn write_portal_name(&mut self, id: Option<u32>) {
        if let Some(id) = id {
            self.extend(b"sqlx_p_");

            itoa::write(&mut *self, id).unwrap();
        }

        self.push(0);
    }
}
