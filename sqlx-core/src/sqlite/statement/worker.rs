use crate::error::Error;
use crate::sqlite::statement::StatementHandle;
use crossbeam_channel::{bounded, unbounded, Sender};
use either::Either;
use libsqlite3_sys::{sqlite3_step, SQLITE_DONE, SQLITE_ROW};
use sqlx_rt::yield_now;
use std::thread;

// Each SQLite connection has a dedicated thread.

// TODO: Tweak this so that we can use a thread pool per pool of SQLite3 connections to reduce
//       OS resource usage. Low priority because a high concurrent load for SQLite3 is very
//       unlikely.

pub(crate) struct StatementWorker {
    tx: Sender<StatementWorkerCommand>,
}

enum StatementWorkerCommand {
    Step {
        statement: StatementHandle,
        tx: Sender<Result<Either<u64, ()>, Error>>,
    },
}

impl StatementWorker {
    pub(crate) fn new() -> Self {
        let (tx, rx) = unbounded();

        thread::spawn(move || {
            for cmd in rx {
                match cmd {
                    StatementWorkerCommand::Step { statement, tx } => {
                        let status = unsafe { sqlite3_step(statement.0.as_ptr()) };

                        let resp = match status {
                            SQLITE_ROW => Ok(Either::Right(())),
                            SQLITE_DONE => Ok(Either::Left(statement.changes())),
                            _ => Err(statement.last_error().into()),
                        };

                        let _ = tx.send(resp);
                    }
                }
            }
        });

        Self { tx }
    }

    pub(crate) async fn step(
        &mut self,
        statement: StatementHandle,
    ) -> Result<Either<u64, ()>, Error> {
        let (tx, rx) = bounded(1);

        self.tx
            .send(StatementWorkerCommand::Step { statement, tx })
            .map_err(|_| Error::WorkerCrashed)?;

        while rx.is_empty() {
            yield_now().await;
        }

        rx.recv().map_err(|_| Error::WorkerCrashed)?
    }
}
