use crate::error::Error;
use crate::sqlite::statement::StatementHandle;
use either::Either;
use libsqlite3_sys::sqlite3_stmt;
use libsqlite3_sys::{sqlite3_step, SQLITE_DONE, SQLITE_ROW};
use sqlx_rt::yield_now;
use std::ptr::null_mut;
use std::sync::atomic::{spin_loop_hint, AtomicI32, AtomicPtr, Ordering};
use std::sync::Arc;
use std::thread::{self, park, spawn, JoinHandle};

const STATE_CLOSE: i32 = -1;

const STATE_READY: i32 = 0;

const STATE_INITIAL: i32 = 1;

// Each SQLite connection has a dedicated thread.

// TODO: Tweak this so that we can use a thread pool per pool of SQLite3 connections to reduce
//       OS resource usage. Low priority because a high concurrent load for SQLite3 is very
//       unlikely.

// TODO: Reduce atomic complexity. There must be a simpler way to do this that doesn't
//       compromise performance.

pub(crate) struct StatementWorker {
    statement: Arc<AtomicPtr<sqlite3_stmt>>,
    status: Arc<AtomicI32>,
    handle: Option<JoinHandle<()>>,
}

impl StatementWorker {
    pub(crate) fn new() -> Self {
        let statement = Arc::new(AtomicPtr::new(null_mut::<sqlite3_stmt>()));
        let status = Arc::new(AtomicI32::new(STATE_INITIAL));

        let handle = spawn({
            let statement = Arc::clone(&statement);
            let status = Arc::clone(&status);

            move || {
                // wait for the first command
                park();

                'run: while status.load(Ordering::Acquire) >= 0 {
                    'statement: loop {
                        match status.load(Ordering::Acquire) {
                            STATE_CLOSE => {
                                // worker has been dropped; get out
                                break 'run;
                            }

                            STATE_READY => {
                                let statement = statement.load(Ordering::Acquire);
                                if statement.is_null() {
                                    // we do not have the statement handle yet
                                    thread::yield_now();
                                    continue;
                                }

                                let v = unsafe { sqlite3_step(statement) };

                                status.store(v, Ordering::Release);

                                if v == SQLITE_DONE {
                                    // when a statement is _done_, we park the thread until
                                    // we need it again
                                    park();
                                    break 'statement;
                                }
                            }

                            _ => {
                                // waits for the receiving end to be ready to receive the rows
                                // this should take less than 1 microsecond under most conditions
                                spin_loop_hint();
                            }
                        }
                    }
                }
            }
        });

        Self {
            handle: Some(handle),
            statement,
            status,
        }
    }

    pub(crate) fn wake(&self) {
        if let Some(handle) = &self.handle {
            handle.thread().unpark();
        }
    }

    pub(crate) fn execute(&self, statement: &StatementHandle) {
        // readies the worker to execute the statement
        // for async-std, this unparks our dedicated thread

        self.statement
            .store(statement.0.as_ptr(), Ordering::Release);
    }

    pub(crate) async fn step(&self, statement: &StatementHandle) -> Result<Either<u64, ()>, Error> {
        // storing <0> as a terminal in status releases the worker
        // to proceed to the next [sqlite3_step] invocation
        self.status.store(STATE_READY, Ordering::Release);

        // we then use a spin loop to wait for this to finish
        // 99% of the time this should be < 1 μs
        let status = loop {
            let status = self
                .status
                .compare_and_swap(STATE_READY, STATE_READY, Ordering::AcqRel);

            if status != STATE_READY {
                break status;
            }

            yield_now().await;
        };

        match status {
            // a row was found
            SQLITE_ROW => Ok(Either::Right(())),

            // reached the end of the query results,
            // emit the # of changes
            SQLITE_DONE => Ok(Either::Left(statement.changes())),

            _ => Err(statement.last_error().into()),
        }
    }

    pub(crate) fn close(&mut self) {
        self.status.store(STATE_CLOSE, Ordering::Release);

        if let Some(handle) = self.handle.take() {
            handle.thread().unpark();
            handle.join().unwrap();
        }
    }
}

impl Drop for StatementWorker {
    fn drop(&mut self) {
        self.close();
    }
}
