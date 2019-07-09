// Reference: https://mariadb.com/kb/en/library/connection

use crate::protocol::{decode::*, error_codes::ErrorCode};
use byteorder::{ByteOrder, LittleEndian};
use bytes::{Bytes, BytesMut};
use failure::{err_msg, Error};
use std::convert::TryFrom;

pub trait Deserialize: Sized {
    fn deserialize(buf: &Bytes) -> Result<Self, Error>;
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Message {
    InitialHandshakePacket(InitialHandshakePacket),
    OkPacket(OkPacket),
    ErrPacket(ErrPacket),
}

bitflags! {
    pub struct Capabilities: u128 {
        const CLIENT_MYSQL = 1;
        const FOUND_ROWS = 1 << 1;
        const CONNECT_WITH_DB = 1 << 3;
        const COMPRESS = 1 << 5;
        const LOCAL_FILES = 1 << 7;
        const IGNORE_SPACE = 1 << 8;
        const CLIENT_PROTOCOL_41 = 1 << 9;
        const CLIENT_INTERACTIVE = 1 << 10;
        const SSL = 1 << 11;
        const TRANSACTIONS = 1 << 12;
        const SECURE_CONNECTION = 1 << 13;
        const MULTI_STATEMENTS = 1 << 16;
        const MULTI_RESULTS = 1 << 17;
        const PS_MULTI_RESULTS = 1 << 18;
        const PLUGIN_AUTH = 1 << 19;
        const CONNECT_ATTRS = 1 << 20;
        const PLUGIN_AUTH_LENENC_CLIENT_DATA = 1 << 21;
        const CLIENT_SESSION_TRACK = 1 << 23;
        const CLIENT_DEPRECATE_EOF = 1 << 24;
        const MARIA_DB_CLIENT_PROGRESS = 1 << 32;
        const MARIA_DB_CLIENT_COM_MULTI = 1 << 33;
        const MARIA_CLIENT_STMT_BULK_OPERATIONS = 1 << 34;
    }
}

bitflags! {
    pub struct FieldDetailFlag: u16 {
        const NOT_NULL = 1;
        const PRIMARY_KEY = 2;
        const UNIQUE_KEY = 4;
        const MULTIPLE_KEY = 8;
        const BLOB = 16;
        const UNSIGNED = 32;
        const ZEROFILL_FLAG = 64;
        const BINARY_COLLATION = 128;
        const ENUM = 256;
        const AUTO_INCREMENT = 512;
        const TIMESTAMP = 1024;
        const SET = 2048;
        const NO_DEFAULT_VALUE_FLAG = 4096;
        const ON_UPDATE_NOW_FLAG = 8192;
        const NUM_FLAG = 32768;
    }
}

bitflags! {
    pub struct ServerStatusFlag: u16 {
        const SERVER_STATUS_IN_TRANS = 1;
        const SERVER_STATUS_AUTOCOMMIT = 2;
        const SERVER_MORE_RESULTS_EXISTS = 8;
        const SERVER_QUERY_NO_GOOD_INDEX_USED = 16;
        const SERVER_QUERY_NO_INDEX_USED = 32;
        const SERVER_STATUS_CURSOR_EXISTS = 64;
        const SERVER_STATUS_LAST_ROW_SENT = 128;
        const SERVER_STATUS_DB_DROPPED = 1 << 8;
        const SERVER_STATUS_NO_BACKSLASH_ESAPES = 1 << 9;
        const SERVER_STATUS_METADATA_CHANGED = 1 << 10;
        const SERVER_QUERY_WAS_SLOW = 1 << 11;
        const SERVER_PS_OUT_PARAMS = 1 << 12;
        const SERVER_STATUS_IN_TRANS_READONLY = 1 << 13;
        const SERVER_SESSION_STATE_CHANGED = 1 << 14;
    }
}

pub enum SessionChangeType {
    SessionTrackSystemVariables = 0,
    SessionTrackSchema = 1,
    SessionTrackStateChange = 2,
    SessionTrackGTIDS = 3,
    SessionTrackTransactionCharacteristics = 4,
    SessionTrackTransactionState = 5,
}

#[derive(Clone, Copy, Debug, PartialEq, TryFromPrimitive)]
#[TryFromPrimitiveType="u8"]
pub enum FieldType {
    MysqlTypeDecimal = 0,
    MysqlTypeTiny = 1,	
    MysqlTypeShort = 2,	
    MysqlTypeLong = 3,	
    MysqlTypeFloat = 4,	
    MysqlTypeDouble = 5,	
    MysqlTypeNull = 6,	
    MysqlTypeTimestamp = 7,	
    MysqlTypeLonglong = 8,	
    MysqlTypeInt24 = 9,	
    MysqlTypeDate = 10,	
    MysqlTypeTime = 11,	
    MysqlTypeDatetime = 12,	
    MysqlTypeYear = 13,	
    MysqlTypeNewdate = 14,	
    MysqlTypeVarchar = 15,	
    MysqlTypeBit = 16,	
    MysqlTypeTimestamp2 = 17,	
    MysqlTypeDatetime2 = 18,	
    MysqlTypeTime2 = 19,	
    MysqlTypeJson = 245,	
    MysqlTypeNewdecimal = 246,	
    MysqlTypeEnum = 247,	
    MysqlTypeSet = 248,	
    MysqlTypeTinyBlob = 249,	
    MysqlTypeMediumBlob = 250,	
    MysqlTypeLongBlob = 251,	
    MysqlTypeBlob = 252,	
    MysqlTypeVarString = 253,	
    MysqlTypeString = 254,	
    MysqlTypeGeometry = 255,	
}

impl Default for Capabilities {
    fn default() -> Self {
        Capabilities::CLIENT_MYSQL
    }
}

impl Default for ServerStatusFlag {
    fn default() -> Self {
        ServerStatusFlag::SERVER_STATUS_IN_TRANS
    }
}

impl Default for FieldType {
    fn default() -> Self {
        FieldType::MysqlTypeDecimal
    }
}

#[derive(Default, Debug)]
pub struct InitialHandshakePacket {
    pub length: u32,
    pub seq_no: u8,
    pub protocol_version: u8,
    pub server_version: Bytes,
    pub connection_id: u32,
    pub auth_seed: Bytes,
    pub capabilities: Capabilities,
    pub collation: u8,
    pub status: ServerStatusFlag,
    pub plugin_data_length: u8,
    pub scramble: Option<Bytes>,
    pub auth_plugin_name: Option<Bytes>,
}

#[derive(Default, Debug)]
pub struct OkPacket {
    pub length: u32,
    pub seq_no: u8,
    pub affected_rows: Option<usize>,
    pub last_insert_id: Option<usize>,
    pub server_status: ServerStatusFlag,
    pub warning_count: u16,
    pub info: Bytes,
    pub session_state_info: Option<Bytes>,
    pub value: Option<Bytes>,
}

#[derive(Default, Debug)]
pub struct ErrPacket {
    pub length: u32,
    pub seq_no: u8,
    pub error_code: ErrorCode,
    pub stage: Option<u8>,
    pub max_stage: Option<u8>,
    pub progress: Option<u32>,
    pub progress_info: Option<Bytes>,
    pub sql_state_marker: Option<Bytes>,
    pub sql_state: Option<Bytes>,
    pub error_message: Option<Bytes>,
}

#[derive(Default, Debug)]
pub struct ColumnPacket {
    pub length: u32,
    pub seq_no: u8,
    pub columns: Option<usize>,
}

pub struct ColumnDefPacket {
    pub length: u32,
    pub seq_no: u8,
    pub catalog: Bytes,
    pub schema: Bytes,
    pub table_alias: Bytes,
    pub table: Bytes,
    pub column_alias: Bytes,
    pub column: Bytes,
    pub length_of_fixed_fields: Option<usize>,
    pub char_set: u16,
    pub max_columns: u32,
    pub field_type: FieldType,
    pub field_details: FieldDetailFlag,
    pub decimals: u8,
}

#[derive(Debug, Default)]
pub struct ResultSet {
    pub columns: Vec<(ColumnPacket, ColumnDefPacket)>,
    pub rows: Vec<Bytes>,
}

impl Message {
    pub fn deserialize(buf: &mut BytesMut) -> Result<Option<Self>, Error> {
        if buf.len() < 4 {
            return Ok(None);
        }

        let length = LittleEndian::read_u24(&buf[0..]) as usize;
        if buf.len() < length + 4 {
            return Ok(None);
        }

        let buf = buf.split_to(length + 4).freeze();
        let _seq_no = [3];
        let tag = buf[4];

        Ok(Some(match tag {
            0xFF => Message::ErrPacket(ErrPacket::deserialize(&buf)?),
            0x00 | 0xFE => Message::OkPacket(OkPacket::deserialize(&buf)?),
            _ => unimplemented!(),
        }))
    }
}

impl Deserialize for InitialHandshakePacket {
    fn deserialize(buf: &Bytes) -> Result<Self, Error> {
        let mut index = 0;

        let length = decode_length(&buf, &mut index)?;
        let seq_no = decode_int_1(&buf, &mut index);

        if seq_no != 0 {
            return Err(err_msg("Squence Number of Initial Handshake Packet is not 0"));
        }

        let protocol_version = decode_int_1(&buf, &mut index);
        let server_version = decode_string_null(&buf, &mut index)?;
        let connection_id = decode_int_4(&buf, &mut index);
        let auth_seed = decode_string_fix(&buf, &mut index, 8);

        // Skip reserved byte
        index += 1;

        let mut capabilities =
            Capabilities::from_bits_truncate(decode_int_2(&buf, &mut index).into());

        let collation = decode_int_1(&buf, &mut index);
        let status =
            ServerStatusFlag::from_bits_truncate(decode_int_2(&buf, &mut index).into());

        capabilities |= Capabilities::from_bits_truncate(
            ((decode_int_2(&buf, &mut index) as u32) << 16).into(),
        );

        let mut plugin_data_length = 0;
        if !(capabilities & Capabilities::PLUGIN_AUTH).is_empty() {
            plugin_data_length = decode_int_1(&buf, &mut index);
        } else {
            // Skip reserve byte
            index += 1;
        }

        // Skip filler
        index += 6;

        if (capabilities & Capabilities::CLIENT_MYSQL).is_empty() {
            capabilities |= Capabilities::from_bits_truncate(
                ((decode_int_4(&buf, &mut index) as u128) << 32).into(),
            );
        } else {
            // Skip filler
            index += 4;
        }

        let mut scramble: Option<Bytes> = None;
        if !(capabilities & Capabilities::SECURE_CONNECTION).is_empty() {
            let len = std::cmp::max(12, plugin_data_length as usize - 9);
            scramble = Some(decode_string_fix(&buf, &mut index, len));
            // Skip reserve byte
            index += 1;
        }

        let mut auth_plugin_name: Option<Bytes> = None;
        if !(capabilities & Capabilities::PLUGIN_AUTH).is_empty() {
            auth_plugin_name = Some(decode_string_null(&buf, &mut index)?);
        }

        Ok(InitialHandshakePacket {
            length,
            seq_no,
            protocol_version,
            server_version,
            connection_id,
            auth_seed,
            capabilities,
            collation,
            status,
            plugin_data_length,
            scramble,
            auth_plugin_name,
        })
    }
}

impl Deserialize for OkPacket {
    fn deserialize(buf: &Bytes) -> Result<Self, Error> {
        let mut index = 0;

        // Packet header
        let length = decode_length(&buf, &mut index)?;
        let seq_no = decode_int_1(&buf, &mut index);

        // Packet body
        let packet_header = decode_int_1(&buf, &mut index);
        if packet_header != 0 && packet_header != 0xFE {
            panic!("Packet header is not 0 or 0xFE for OkPacket");
        }

        let affected_rows = decode_int_lenenc(&buf, &mut index);
        let last_insert_id = decode_int_lenenc(&buf, &mut index);
        let server_status =
            ServerStatusFlag::from_bits_truncate(decode_int_2(&buf, &mut index).into());
        let warning_count = decode_int_2(&buf, &mut index);

        // Assuming CLIENT_SESSION_TRACK is unsupported
        let session_state_info = None;
        let value = None;

        let info = Bytes::from(&buf[index..]);

        Ok(OkPacket {
            length,
            seq_no,
            affected_rows,
            last_insert_id,
            server_status,
            warning_count,
            info,
            session_state_info,
            value,
        })
    }
}

impl Deserialize for ErrPacket {
    fn deserialize(buf: &Bytes) -> Result<Self, Error> {
        let mut index = 0;

        let length = decode_length(&buf, &mut index)?;
        let seq_no = decode_int_1(&buf, &mut index);

        let packet_header = decode_int_1(&buf, &mut index);
        if packet_header != 0xFF {
            panic!("Packet header is not 0xFF for ErrPacket");
        }

        let error_code = ErrorCode::try_from(decode_int_2(&buf, &mut index))?;

        let mut stage = None;
        let mut max_stage = None;
        let mut progress = None;
        let mut progress_info = None;

        let mut sql_state_marker = None;
        let mut sql_state = None;
        let mut error_message = None;

        // Progress Reporting
        if error_code as u16 == 0xFFFF {
            stage = Some(decode_int_1(buf, &mut index));
            max_stage = Some(decode_int_1(buf, &mut index));
            progress = Some(decode_int_3(buf, &mut index));
            progress_info = Some(decode_string_lenenc(&buf, &mut index));
        } else {
            if buf[index] == b'#' {
                sql_state_marker = Some(decode_string_fix(buf, &mut index, 1));
                sql_state = Some(decode_string_fix(buf, &mut index, 5));
                error_message = Some(decode_string_eof(buf, &mut index));
            } else {
                error_message = Some(decode_string_eof(buf, &mut index));
            }
        }

        Ok(ErrPacket {
            length,
            seq_no,
            error_code,
            stage,
            max_stage,
            progress,
            progress_info,
            sql_state_marker,
            sql_state,
            error_message,
        })
    }
}

impl Deserialize for ColumnPacket {
    fn deserialize(buf: &Bytes) -> Result<Self, Error> {
        let mut index = 0;

        let length = decode_length(&buf, &mut index)?;
        let seq_no = decode_int_1(&buf, &mut index);
        let columns = decode_int_lenenc(&buf, &mut index);

        Ok(ColumnPacket {
            length,
            seq_no,
            columns,
        })
    }
}

impl Deserialize for ColumnDefPacket {
    fn deserialize(buf: &Bytes) -> Result<Self, Error> {
        let mut index = 0;

        let length = decode_length(&buf, &mut index)?;
        let seq_no = decode_int_1(&buf, &mut index);

        let catalog = decode_string_lenenc(&buf, &mut index);
        let schema = decode_string_lenenc(&buf, &mut index);
        let table_alias = decode_string_lenenc(&buf, &mut index);
        let table = decode_string_lenenc(&buf, &mut index);
        let column_alias = decode_string_lenenc(&buf, &mut index);
        let column = decode_string_lenenc(&buf, &mut index);
        let length_of_fixed_fields = decode_int_lenenc(&buf, &mut index);
        let char_set = decode_int_2(&buf, &mut index);
        let max_columns = decode_int_4(&buf, &mut index);
        let field_type = FieldType::try_from(decode_int_1(&buf, &mut index))?;
        let field_details = FieldDetailFlag::from_bits_truncate(decode_int_2(&buf, &mut index));
        let decimals = decode_int_1(&buf, &mut index);

        // Skip last two unused bytes
        // index += 2;

        Ok(ColumnDefPacket {
            length,
            seq_no,
            catalog,
            schema,
            table_alias,
            table,
            column_alias,
            column,
            length_of_fixed_fields,
            char_set,
            max_columns,
            field_type,
            field_details,
            decimals,
        })
    }
}

//impl Deserialize for ResultSet {
//    fn deserialize(buf: &Bytes) -> Result<Self, Error> {
//        let mut index = 0;
//
//        let length = decode_length(&buf, &mut index)?;
//        let seq_no = decode_int_1(&buf, &mut index);
//
//        let column_packet = ColumnPacket::deserialize(&but)?;
//
//        let column_definitions = if let Some(columns) = column_packet.columns {
//            (0..columns).map(|_| {
//                ColumnDefPacket::deserialize()
//            })
//        };
//
//        Ok(ResultSet::default())
//    }
//}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_decodes_capabilities() {
        let buf = BytesMut::from(b"\xfe\xf7".to_vec());
        let mut index = 0;
        Capabilities::from_bits_truncate(decode_int_2(&buf.freeze(), &mut index).into());
    }

    #[test]
    fn it_decodes_errpacket_real() -> Result<(), Error> {
        let buf = BytesMut::from(b"!\0\0\x01\xff\x84\x04#08S01Got packets out of order".to_vec());
        let _message = ErrPacket::deserialize(&buf.freeze())?;

        Ok(())
    }

    #[test]
    fn it_decodes_initialhandshakepacket() -> Result<(), Error> {
        let buf = BytesMut::from(
            b"\
        n\0\0\
        \0\
        \n\
        5.5.5-10.4.6-MariaDB-1:10.4.6+maria~bionic\0\
        \x13\0\0\0\
        ?~~|vZAu\
        \0\
        \xfe\xf7\
        \x08\
        \x02\0\
        \xff\x81\
        \x15\
        \0\0\0\0\0\0\
        \x07\0\0\0\
        JQ8cihP4Q}Dx\
        \0\
        mysql_native_password\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"
                .to_vec(),
        );

        let _message = InitialHandshakePacket::deserialize(&buf.freeze())?;

        Ok(())
    }

    #[test]
    fn it_decodes_okpacket() -> Result<(), Error> {
        let buf = BytesMut::from(
            b"\
        \x0F\x00\x00\
        \x01\
        \x00\
        \xFB\
        \xFB\
        \x01\x01\
        \x00\x00\
        info\
        "
            .to_vec(),
        );

        let message = OkPacket::deserialize(&buf.freeze())?;

        assert_eq!(message.affected_rows, None);
        assert_eq!(message.last_insert_id, None);
        assert!(!(message.server_status & ServerStatusFlag::SERVER_STATUS_IN_TRANS).is_empty());
        assert_eq!(message.warning_count, 0);
        assert_eq!(message.info, b"info".to_vec());

        Ok(())
    }

    #[test]
    fn it_decodes_errpacket() -> Result<(), Error> {
        let buf = BytesMut::from(
            b"\
        \x0F\x00\x00\
        \x01\
        \xFF\
        \xEA\x03\
        #\
        HY000\
        NO\
        "
            .to_vec(),
        );

        let message = ErrPacket::deserialize(&buf.freeze())?;

        assert_eq!(message.error_code, 1002);
        assert_eq!(message.sql_state_marker, Some(Bytes::from(b"#".to_vec())));
        assert_eq!(message.sql_state, Some(Bytes::from(b"HY000".to_vec())));
        assert_eq!(message.error_message, Some(Bytes::from(b"NO".to_vec())));

        Ok(())
    }
}
