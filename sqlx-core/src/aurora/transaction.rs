use futures_core::future::BoxFuture;

use crate::aurora::connection::AuroraConnection;
use crate::aurora::error::AuroraDatabaseError;
use crate::aurora::Aurora;
use crate::error::Error;
use crate::transaction::TransactionManager;

use rusoto_rds_data::{
    BeginTransactionRequest, CommitTransactionRequest, RdsData, RollbackTransactionRequest,
};

/// Implementation of [`TransactionManager`] for Aurora.
pub struct AuroraTransactionManager;

impl TransactionManager for AuroraTransactionManager {
    type Database = Aurora;

    fn begin(conn: &mut AuroraConnection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            let request = BeginTransactionRequest {
                database: conn.database.clone(),
                resource_arn: conn.resource_arn.clone(),
                schema: conn.schema.clone(),
                secret_arn: conn.secret_arn.clone(),
            };

            let response = conn
                .client
                .begin_transaction(request)
                .await
                .map_err(AuroraDatabaseError)?;

            // TODO: when can `BeginTransactionResponse.transaction_id` be `None`
            // and what do we do in that instance?
            if let Some(id) = response.transaction_id {
                conn.transaction_ids.push(id);
            }

            Ok(())
        })
    }

    fn commit(conn: &mut AuroraConnection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            if let Some(id) = conn.transaction_ids.last() {
                let request = CommitTransactionRequest {
                    resource_arn: conn.resource_arn.clone(),
                    secret_arn: conn.secret_arn.clone(),
                    transaction_id: id.clone(),
                };

                let _response = conn
                    .client
                    .commit_transaction(request)
                    .await
                    .map_err(AuroraDatabaseError)?;

                // TODO: what does `CommitTransactionResponse.transaction_status`
                // signify and how should we handle it?

                conn.transaction_ids.pop();
            }

            Ok(())
        })
    }

    fn rollback(conn: &mut AuroraConnection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            if let Some(id) = conn.transaction_ids.last() {
                let request = RollbackTransactionRequest {
                    resource_arn: conn.resource_arn.clone(),
                    secret_arn: conn.secret_arn.clone(),
                    transaction_id: id.clone(),
                };

                let _response = conn
                    .client
                    .rollback_transaction(request)
                    .await
                    .map_err(AuroraDatabaseError)?;

                // TODO: what does `RollbackTransactionRequest.transaction_status`
                // signify and how should we handle it?

                conn.transaction_ids.pop();
            }

            Ok(())
        })
    }

    fn start_rollback(conn: &mut AuroraConnection) {
        // TODO: Not sure how to implement this
    }
}
