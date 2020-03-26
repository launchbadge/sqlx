use std::net::{Ipv4Addr, Ipv6Addr};

use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};

use crate::decode::Decode;
use crate::encode::Encode;
use crate::postgres::protocol::TypeId;
use crate::postgres::types::PgTypeInfo;
use crate::postgres::value::PgValue;
use crate::postgres::{PgData, Postgres};
use crate::types::Type;
use crate::Error;

#[cfg(windows)]
const AF_INET: u8 = 2;
// Maybe not used, but defining to follow Rust's libstd/net/sys
#[cfg(redox)]
const AF_INET: u8 = 1;
#[cfg(not(any(windows, redox)))]
const AF_INET: u8 = libc::AF_INET as u8;

const PGSQL_AF_INET: u8 = AF_INET;
const PGSQL_AF_INET6: u8 = AF_INET + 1;

const INET_TYPE: u8 = 0;
const CIDR_TYPE: u8 = 1;

impl Type<Postgres> for IpNetwork {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::INET, "INET")
    }
}

impl Type<Postgres> for [IpNetwork] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_INET, "INET[]")
    }
}

impl Encode<Postgres> for IpNetwork {
    fn encode(&self, buf: &mut Vec<u8>) {
        match self {
            IpNetwork::V4(net) => {
                buf.push(PGSQL_AF_INET);
                buf.push(net.prefix());
                buf.push(INET_TYPE);
                buf.push(4);
                buf.extend_from_slice(&net.ip().octets());
            }
            IpNetwork::V6(net) => {
                buf.push(PGSQL_AF_INET6);
                buf.push(net.prefix());
                buf.push(INET_TYPE);
                buf.push(16);
                buf.extend_from_slice(&net.ip().octets());
            }
        }
    }

    fn size_hint(&self) -> usize {
        match self {
            IpNetwork::V4(_) => 8,
            IpNetwork::V6(_) => 20,
        }
    }
}

impl<'de> Decode<'de, Postgres> for IpNetwork {
    fn decode(value: PgValue<'de>) -> crate::Result<Self> {
        match value.try_get()? {
            PgData::Binary(buf) => decode(buf),
            PgData::Text(s) => s.parse().map_err(crate::Error::decode),
        }
    }
}

fn decode(bytes: &[u8]) -> crate::Result<IpNetwork> {
    if bytes.len() < 8 {
        return Err(Error::Decode("Input too short".into()));
    }

    let af = bytes[0];
    let prefix = bytes[1];
    let net_type = bytes[2];
    let len = bytes[3];

    if net_type == INET_TYPE || net_type == CIDR_TYPE {
        if af == PGSQL_AF_INET && bytes.len() == 8 && len == 4 {
            let inet = Ipv4Network::new(
                Ipv4Addr::new(bytes[4], bytes[5], bytes[6], bytes[7]),
                prefix,
            )
            .map_err(Error::decode)?;

            return Ok(IpNetwork::V4(inet));
        }

        if af == PGSQL_AF_INET6 && bytes.len() == 20 && len == 16 {
            let inet = Ipv6Network::new(
                Ipv6Addr::from([
                    bytes[4], bytes[5], bytes[6], bytes[7], bytes[8], bytes[9], bytes[10],
                    bytes[11], bytes[12], bytes[13], bytes[14], bytes[15], bytes[16], bytes[17],
                    bytes[18], bytes[19],
                ]),
                prefix,
            )
            .map_err(Error::decode)?;

            return Ok(IpNetwork::V6(inet));
        }
    }

    return Err(Error::Decode("Invalid input".into()));
}
