use sqlx_core::io::Serialize;
use sqlx_core::Result;

use crate::io::PgWriteExt;
use crate::protocol::frontend::{PortalRef, StatementId};

/// Target a command at a portal *or* statement.
/// Used by [`Describe`] and [`Close`].
#[derive(Debug)]
pub(crate) enum Target {
    Portal(PortalRef),
    Statement(StatementId),
}

impl Serialize<'_> for Target {
    fn serialize_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<()> {
        buf.write_len_prefixed(|buf| {
            match self {
                Self::Portal(portal) => {
                    buf.push(b'P');
                    portal.serialize(buf)?;
                }

                Self::Statement(statement) => {
                    buf.push(b'S');
                    statement.serialize(buf)?;
                }
            }

            Ok(())
        })
    }
}
