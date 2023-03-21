use std::ffi::c_void;
use std::os::raw::c_int;
use std::slice;
use std::sync::{Condvar, Mutex};

use libsqlite3_sys::{sqlite3, sqlite3_unlock_notify, SQLITE_OK};

use crate::sqlite::SqliteError;

// Wait for unlock notification (https://www.sqlite.org/unlock_notify.html)
pub unsafe fn wait(conn: *mut sqlite3) -> Result<(), SqliteError> {
    let notify = Notify::new();

    if sqlite3_unlock_notify(
        conn,
        Some(unlock_notify_cb),
        &notify as *const Notify as *mut Notify as *mut _,
    ) != SQLITE_OK
    {
        return Err(SqliteError::new(conn));
    }

    notify.wait();

    Ok(())
}

unsafe extern "C" fn unlock_notify_cb(ptr: *mut *mut c_void, len: c_int) {
    let ptr = ptr as *mut &Notify;
    let slice = slice::from_raw_parts(ptr, len as usize);

    for notify in slice {
        notify.fire();
    }
}

struct Notify {
    mutex: Mutex<bool>,
    condvar: Condvar,
}

impl Notify {
    fn new() -> Self {
        Self {
            mutex: Mutex::new(false),
            condvar: Condvar::new(),
        }
    }

    fn wait(&self) {
        let _guard = self
            .condvar
            .wait_while(self.mutex.lock().unwrap(), |fired| !*fired)
            .unwrap();
    }

    fn fire(&self) {
        *self.mutex.lock().unwrap() = true;
        self.condvar.notify_one();
    }
}
