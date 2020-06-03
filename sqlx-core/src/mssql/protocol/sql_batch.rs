use crate::io::Encode;
use crate::mssql::io::MsSqlBufMutExt;

const HEADER_TRANSACTION_DESCRIPTOR: u16 = 0x00_02;

#[derive(Debug)]
pub(crate) struct SqlBatch<'a> {
    pub(crate) sql: &'a str,
}

impl Encode<'_> for SqlBatch<'_> {
    fn encode_with(&self, buf: &mut Vec<u8>, _: ()) {
        // ALL_HEADERS -> TotalLength
        buf.extend(&(4_u32 + 18).to_le_bytes()); // 4 + 18

        // [Header] Transaction Descriptor
        //  SQL_BATCH messages require this header
        //  contains information regarding number of outstanding requests for MARS
        buf.extend(&18_u32.to_le_bytes()); // 4 + 2 + 8 + 4
        buf.extend(&HEADER_TRANSACTION_DESCRIPTOR.to_le_bytes());

        // [TransactionDescriptor] a number that uniquely identifies the current transaction
        // TODO: use this once we support transactions, it will be given to us from the
        //       server ENVCHANGE event
        buf.extend(&0_u64.to_le_bytes());

        // [OutstandingRequestCount] Number of active requests to MSSQL from the
        //                           same connection
        // NOTE: Long-term when we support MARS we need to connect this value correctly
        buf.extend(&(1_u32.to_le_bytes()));

        // SQLText
        buf.put_utf16_str(self.sql);
    }
}
