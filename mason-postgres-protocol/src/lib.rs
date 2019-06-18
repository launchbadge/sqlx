//! https://www.postgresql.org/docs/devel/protocol.html
#![feature(non_exhaustive)]

mod decode;
mod encode;
mod message;
mod notice_response;
mod ready_for_query;

pub use self::{
    decode::Decode,
    encode::Encode,
    message::Message,
    notice_response::{NoticeResponse, Severity},
    ready_for_query::{ReadyForQuery, TransactionStatus},
};
