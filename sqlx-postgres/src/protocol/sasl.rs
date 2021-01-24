use sqlx_core::io::Serialize;
use sqlx_core::io::WriteExt;
use sqlx_core::Result;

use crate::io::PgBufMutExt;

#[derive(Debug)]
pub struct SaslInitialResponse {
    pub response: String,
    pub plus: bool,
}

impl Serialize<'_, ()> for SaslInitialResponse {
    fn serialize_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<()> {
        buf.push(b'p');
        buf.write_length_prefixed(|buf| {
            // name of the SASL authentication mechanism that the client selected
            buf.write_str_nul(if self.plus { "SCRAM-SHA-256-PLUS" } else { "SCRAM-SHA-256" });

            buf.extend(&(self.response.as_bytes().len() as i32).to_be_bytes());
            buf.extend(self.response.as_bytes());
        });

        Ok(())
    }
}

#[derive(Debug)]
pub struct SaslResponse<'a>(pub &'a str);

impl Serialize<'_, ()> for SaslResponse<'_> {
    fn serialize_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<()> {
        buf.push(b'p');
        buf.write_length_prefixed(|buf| {
            buf.extend(self.0.as_bytes());
        });

        Ok(())
    }
}
