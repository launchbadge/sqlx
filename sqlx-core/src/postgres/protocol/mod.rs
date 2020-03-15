//! Low level Postgres protocol. Defines the encoding and decoding of the messages communicated
//! to and from the database server.
#![allow(unused)]

mod type_format;
mod type_id;

pub use type_format::TypeFormat;
pub use type_id::TypeId;

// REQUESTS
mod bind;
mod describe;
mod execute;
mod parse;
mod password_message;
mod query;
mod sasl;
#[cfg_attr(not(feature = "tls"), allow(unused_imports, dead_code))]
mod ssl_request;
mod startup_message;
mod statement;
mod sync;
mod terminate;

pub(crate) use bind::Bind;
pub(crate) use describe::Describe;
pub(crate) use execute::Execute;
pub(crate) use parse::Parse;
pub(crate) use password_message::PasswordMessage;
pub(crate) use query::Query;
pub(crate) use sasl::{hi, SaslInitialResponse, SaslResponse};
#[cfg_attr(not(feature = "tls"), allow(unused_imports, dead_code))]
pub(crate) use ssl_request::SslRequest;
pub(crate) use startup_message::StartupMessage;
pub(crate) use statement::StatementId;
pub(crate) use sync::Sync;
pub(crate) use terminate::Terminate;

// RESPONSES
mod authentication;
mod backend_key_data;
mod command_complete;
mod data_row;
mod notification_response;
mod parameter_description;
mod ready_for_query;
mod response;
mod row_description;

mod message;

pub(crate) use authentication::{
    Authentication, AuthenticationMd5, AuthenticationSasl, AuthenticationSaslContinue,
};
pub(crate) use backend_key_data::BackendKeyData;
pub(crate) use command_complete::CommandComplete;
pub(crate) use data_row::DataRow;
pub(crate) use message::Message;
pub(crate) use notification_response::NotificationResponse;
pub(crate) use parameter_description::ParameterDescription;
pub(crate) use ready_for_query::ReadyForQuery;
pub(crate) use response::Response;
pub(crate) use row_description::{Field, RowDescription};

pub(crate) trait Write {
    fn write(&self, buf: &mut Vec<u8>);
}
