//! Low level Postgres protocol. Defines the encoding and decoding of the messages communicated
//! to and from the database server.

mod type_format;
mod type_id;

pub use type_format::TypeFormat;
pub use type_id::TypeId;

// REQUESTS
mod bind;
mod cancel_request;
mod describe;
mod encode;
mod execute;
mod flush;
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

pub use bind::Bind;
pub use cancel_request::CancelRequest;
pub use describe::Describe;
pub use encode::Encode;
pub use execute::Execute;
pub use flush::Flush;
pub use parse::Parse;
pub use password_message::PasswordMessage;
pub use query::Query;
pub use sasl::{hi, SaslInitialResponse, SaslResponse};
pub use ssl_request::SslRequest;
pub use startup_message::StartupMessage;
pub use statement::StatementId;
pub use sync::Sync;
pub use terminate::Terminate;

// RESPONSES
mod authentication;
mod backend_key_data;
mod command_complete;
mod data_row;
mod decode;
mod notification_response;
mod parameter_description;
mod parameter_status;
mod ready_for_query;
mod response;
mod row_description;

mod message;

pub use authentication::{
    Authentication, AuthenticationMd5, AuthenticationSasl, AuthenticationSaslContinue,
};
pub use backend_key_data::BackendKeyData;
pub use command_complete::CommandComplete;
pub use data_row::DataRow;
pub use decode::Decode;
pub use message::Message;
pub use notification_response::NotificationResponse;
pub use parameter_description::ParameterDescription;
pub use parameter_status::ParameterStatus;
pub use ready_for_query::ReadyForQuery;
pub use response::{Response, Severity};
pub use row_description::{Field, RowDescription};
