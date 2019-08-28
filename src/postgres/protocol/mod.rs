//! Low level PostgreSQL protocol. Defines the encoding and decoding of the messages communicated
//! to and from the database server.

mod bind;
mod cancel_request;
mod close;
mod copy_data;
mod copy_done;
mod copy_fail;
mod describe;
mod encode;
mod execute;
mod flush;
mod parse;
mod password_message;
mod query;
mod startup_message;
mod sync;
mod terminate;

// TODO: mod gss_enc_request;
// TODO: mod gss_response;
// TODO: mod sasl_initial_response;
// TODO: mod sasl_response;
// TODO: mod ssl_request;

pub use self::{
    bind::Bind,
    cancel_request::CancelRequest,
    close::Close,
    copy_data::CopyData,
    copy_done::CopyDone,
    copy_fail::CopyFail,
    describe::Describe,
    encode::Encode,
    execute::Execute,
    flush::Flush,
    parse::Parse,
    password_message::PasswordMessage,
    query::Query,
    startup_message::StartupMessage,
    sync::Sync,
    terminate::Terminate,
};

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

// TODO: Audit backend protocol

mod message;

pub use self::{
    authentication::Authentication, backend_key_data::BackendKeyData,
    command_complete::CommandComplete, data_row::DataRow, decode::Decode, message::Message,
    notification_response::NotificationResponse, parameter_description::ParameterDescription,
    parameter_status::ParameterStatus, ready_for_query::ReadyForQuery, response::Response,
};
