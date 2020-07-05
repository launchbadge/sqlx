use crate::error::Error;
use crate::mssql::protocol::col_meta_data::Flags;
use crate::mssql::protocol::done::Status;
use crate::mssql::protocol::message::Message;
use crate::mssql::protocol::packet::PacketType;
use crate::mssql::protocol::rpc::{OptionFlags, Procedure, RpcRequest};
use crate::mssql::{Mssql, MssqlArguments, MssqlConnection};
use crate::statement::StatementInfo;
use either::Either;
use once_cell::sync::Lazy;
use regex::Regex;

pub async fn describe(
    conn: &mut MssqlConnection,
    query: &str,
) -> Result<StatementInfo<Mssql>, Error> {
    // [sp_prepare] will emit the column meta data
    // small issue is that we need to declare all the used placeholders with a "fallback" type
    // we currently use regex to collect them; false positives are *okay* but false
    // negatives would break the query
    let proc = Either::Right(Procedure::Prepare);

    // NOTE: this does not support unicode identifiers; as we don't even support
    //       named parameters (yet) this is probably fine, for now

    static PARAMS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"@p[[:alnum:]]+").unwrap());

    let mut params = String::new();
    let mut num_params = 0;

    for m in PARAMS_RE.captures_iter(query) {
        if !params.is_empty() {
            params.push_str(",");
        }

        params.push_str(&m[0]);

        // NOTE: this means that a query! of `SELECT @p1` will have the macros believe
        //       it will return nvarchar(1); this is a greater issue with `query!` that we
        //       we need to circle back to. This doesn't happen much in practice however.
        params.push_str(" nvarchar(1)");

        num_params += 1;
    }

    let params = if params.is_empty() {
        None
    } else {
        Some(&*params)
    };

    let mut args = MssqlArguments::default();

    args.declare("", 0_i32);
    args.add_unnamed(params);
    args.add_unnamed(query);
    args.add_unnamed(0x0001_i32); // 1 = SEND_METADATA

    conn.stream.write_packet(
        PacketType::Rpc,
        RpcRequest {
            transaction_descriptor: conn.stream.transaction_descriptor,
            arguments: &args,
            procedure: proc,
            options: OptionFlags::empty(),
        },
    );

    conn.stream.flush().await?;
    conn.stream.wait_until_ready().await?;
    conn.stream.pending_done_count += 1;

    loop {
        match conn.stream.recv_message().await? {
            Message::DoneProc(done) | Message::Done(done) => {
                if !done.status.contains(Status::DONE_MORE) {
                    // done with prepare
                    conn.stream.handle_done(&done);
                    break;
                }
            }

            _ => {}
        }
    }

    let mut nullable = Vec::with_capacity(conn.stream.columns.len());

    for col in conn.stream.columns.iter() {
        nullable.push(Some(col.flags.contains(Flags::NULLABLE)));
    }

    Ok(StatementInfo {
        parameters: Some(Either::Right(num_params)),
        columns: (*conn.stream.columns).clone(),
        nullable,
    })
}
