//! SQLite custom memory allocator support.
//!
//! This module provides functionality to configure custom memory allocators for SQLite.
//!
//! **Important**: Memory allocator configuration must be done before creating any SQLite
//! database connections. Once a connection has been established, the memory allocator
//! cannot be changed.
//!
//! # Examples
//!
//! ## Basic System Allocator Wrapper
//!
//! ```rust,no_run
//! use sqlx_sqlite::memory::{SqliteMemoryAllocator, configure_memory_allocator};
//! use std::alloc::{GlobalAlloc, Layout, System};
//! use std::ptr;
//!
//! // Define a custom allocator
//! struct MyCustomAllocator;
//!
//! unsafe impl SqliteMemoryAllocator for MyCustomAllocator {
//!     unsafe fn malloc(&mut self, size: i32) -> *mut std::ffi::c_void {
//!         if size <= 0 {
//!             return ptr::null_mut();
//!         }
//!         
//!         let layout = Layout::from_size_align_unchecked(size as usize, 8);
//!         System.alloc(layout) as *mut std::ffi::c_void
//!     }
//!
//!     unsafe fn free(&mut self, ptr: *mut std::ffi::c_void) {
//!         if !ptr.is_null() {
//!             // Note: In a real implementation, you'd need to track the layout
//!             // This is just an example
//!             System.dealloc(ptr as *mut u8, Layout::from_size_align_unchecked(1, 8));
//!         }
//!     }
//!
//!     unsafe fn realloc(&mut self, ptr: *mut std::ffi::c_void, size: i32) -> *mut std::ffi::c_void {
//!         if size <= 0 {
//!             self.free(ptr);
//!             return ptr::null_mut();
//!         }
//!
//!         if ptr.is_null() {
//!             return self.malloc(size);
//!         }
//!
//!         // Simplified realloc - in practice you'd want to preserve data
//!         self.free(ptr);
//!         self.malloc(size)
//!     }
//!
//!     unsafe fn size(&mut self, ptr: *mut std::ffi::c_void) -> i32 {
//!         // In a real implementation, you'd track allocation sizes
//!         0
//!     }
//!
//!     unsafe fn roundup(&mut self, size: i32) -> i32 {
//!         // Round up to next multiple of 8
//!         (size + 7) & !7
//!     }
//!
//!     unsafe fn init(&mut self, _app_data: *mut std::ffi::c_void) -> i32 {
//!         // Initialization successful
//!         0 // SQLITE_OK
//!     }
//!
//!     unsafe fn shutdown(&mut self, _app_data: *mut std::ffi::c_void) {
//!         // Cleanup if needed
//!     }
//! }
//!
//! // Configure the allocator before creating any connections
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! configure_memory_allocator(MyCustomAllocator)?;
//!
//! // Now create connections as usual
//! # Ok(())
//! # }
//! ```
//!
//! ## Using Jemalloc for High-Performance Applications
//!
//! For applications requiring high-performance memory allocation, you can use
//! [jemalloc](https://github.com/jemalloc/jemalloc) via the
//! [`jemallocator`](https://crates.io/crates/jemallocator) crate:
//!
//! ```toml
//! [dependencies]
//! jemallocator = "0.5"
//! ```
//!
//! ```rust,no_run
//! use sqlx_sqlite::memory::{SqliteMemoryAllocator, configure_memory_allocator};
//! use jemallocator::Jemalloc;
//! use std::alloc::{GlobalAlloc, Layout};
//! use std::ffi::c_void;
//! use std::os::raw::c_int;
//! use std::ptr;
//! use std::sync::atomic::{AtomicI64, Ordering};
//!
//! struct JemallocSqliteAllocator {
//!     allocations: AtomicI64,
//! }
//!
//! impl JemallocSqliteAllocator {
//!     fn new() -> Self {
//!         Self {
//!             allocations: AtomicI64::new(0),
//!         }
//!     }
//! }
//!
//! unsafe impl SqliteMemoryAllocator for JemallocSqliteAllocator {
//!     unsafe fn malloc(&mut self, size: c_int) -> *mut c_void {
//!         if size <= 0 {
//!             return ptr::null_mut();
//!         }
//!         
//!         // Use C malloc directly - jemalloc will be used if it's the global allocator
//!         extern "C" { fn malloc(size: usize) -> *mut c_void; }
//!         let ptr = malloc(size as usize);
//!         if !ptr.is_null() {
//!             self.allocations.fetch_add(1, Ordering::Relaxed);
//!         }
//!         ptr
//!     }
//!
//!     unsafe fn free(&mut self, ptr: *mut c_void) {
//!         if !ptr.is_null() {
//!             extern "C" { fn free(ptr: *mut c_void); }
//!             free(ptr);
//!         }
//!     }
//!
//!     unsafe fn realloc(&mut self, ptr: *mut c_void, size: c_int) -> *mut c_void {
//!         if size <= 0 {
//!             self.free(ptr);
//!             return ptr::null_mut();
//!         }
//!
//!         if ptr.is_null() {
//!             return self.malloc(size);
//!         }
//!
//!         // Use C realloc directly - preserves data and uses jemalloc
//!         extern "C" { fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void; }
//!         realloc(ptr, size as usize)
//!     }
//!
//!     unsafe fn size(&mut self, ptr: *mut c_void) -> c_int {
//!         if ptr.is_null() { return 0; }
//!         extern "C" { fn malloc_usable_size(ptr: *mut c_void) -> usize; }
//!         malloc_usable_size(ptr) as c_int
//!     }
//!
//!     unsafe fn roundup(&mut self, size: c_int) -> c_int {
//!         if size <= 0 { return 8; }
//!         // Test allocation to get jemalloc's actual rounding
//!         extern "C" {
//!             fn malloc(size: usize) -> *mut c_void;
//!             fn malloc_usable_size(ptr: *mut c_void) -> usize;
//!             fn free(ptr: *mut c_void);
//!         }
//!         let test_ptr = malloc(size as usize);
//!         if test_ptr.is_null() { return (size + 7) & !7; }
//!         let actual = malloc_usable_size(test_ptr) as c_int;
//!         free(test_ptr);
//!         actual
//!     }
//!
//!     unsafe fn init(&mut self, _app_data: *mut c_void) -> c_int {
//!         0 // SQLITE_OK
//!     }
//!
//!     unsafe fn shutdown(&mut self, _app_data: *mut c_void) {
//!         // Cleanup
//!     }
//! }
//!
//! // First, set jemalloc as the global allocator for your application
//! #[global_allocator]
//! static GLOBAL: Jemalloc = Jemalloc;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Configure jemalloc for SQLite before any database operations
//! configure_memory_allocator(JemallocSqliteAllocator::new())?;
//!
//! // Use SQLx as normal - now SQLite will use jemalloc for all allocations
//! # Ok(())
//! # }
//! ```

use libsqlite3_sys::{sqlite3_config, sqlite3_mem_methods, SQLITE_CONFIG_MALLOC, SQLITE_OK};
use std::ffi::c_void;
use std::os::raw::c_int;
use std::ptr;
use std::sync::{Mutex, OnceLock};

/// Trait for implementing custom SQLite memory allocators.
///
/// All methods in this trait correspond directly to the function pointers in
/// SQLite's `sqlite3_mem_methods` structure. Implementors must provide safe
/// implementations of these memory management functions.
///
/// **Safety**: All methods in this trait are `unsafe` because they deal with raw
/// memory management. Implementations must ensure:
///
/// - `malloc` returns properly aligned memory for the requested size
/// - `free` only frees memory that was allocated by this allocator
/// - `realloc` properly handles data preservation and edge cases
/// - `size` returns accurate size information for allocated blocks
///
/// See SQLite documentation for detailed requirements:
/// <https://www.sqlite.org/c3ref/mem_methods.html>
///
/// # Safety
///
/// This trait is unsafe because it requires implementing raw memory management
/// functions that SQLite will call directly. Incorrect implementations can lead
/// to memory corruption, crashes, or undefined behavior.
pub unsafe trait SqliteMemoryAllocator: Send + 'static {
    /// Allocate `size` bytes of memory and return a pointer to it.
    ///
    /// Should return null if allocation fails or if `size` is <= 0.
    /// The returned memory should be suitably aligned for any use.
    ///
    /// # Safety
    ///
    /// The returned pointer must be valid for reads and writes of `size` bytes.
    /// The memory must remain valid until freed with `free` or `realloc`.
    unsafe fn malloc(&mut self, size: c_int) -> *mut c_void;

    /// Free a block of memory that was allocated by `malloc` or `realloc`.
    ///
    /// This method should handle null pointers gracefully (no-op).
    ///
    /// # Safety
    ///
    /// `ptr` must be null or a valid pointer returned by this allocator's
    /// `malloc` or `realloc` methods that has not been freed already.
    unsafe fn free(&mut self, ptr: *mut c_void);

    /// Change the size of a memory allocation.
    ///
    /// If `ptr` is null, this should behave like `malloc(size)`.
    /// If `size` is 0, this should behave like `free(ptr)` and return null.
    /// Otherwise, return a pointer to a memory block of at least `size` bytes,
    /// preserving the contents of the original allocation up to the minimum
    /// of the old and new sizes.
    ///
    /// # Safety
    ///
    /// `ptr` must be null or a valid pointer returned by this allocator.
    /// The returned pointer must be valid for reads and writes of `size` bytes.
    unsafe fn realloc(&mut self, ptr: *mut c_void, size: c_int) -> *mut c_void;

    /// Return the size of a memory allocation.
    ///
    /// This should return the usable size of the memory block pointed to by `ptr`,
    /// which must have been allocated by this allocator.
    ///
    /// # Safety
    ///
    /// `ptr` must be a valid pointer returned by this allocator's `malloc`
    /// or `realloc` methods that has not been freed.
    unsafe fn size(&mut self, ptr: *mut c_void) -> c_int;

    /// Round up an allocation request to the next valid allocation size.
    ///
    /// This is used by SQLite to determine good allocation sizes and to
    /// avoid frequent reallocations for small size increases.
    ///
    /// # Safety
    ///
    /// This function should be safe to call with any `size` value.
    /// It must return a value >= `size`.
    unsafe fn roundup(&mut self, size: c_int) -> c_int;

    /// Initialize the memory allocator.
    ///
    /// This is called once when the allocator is installed.
    /// Return 0 (SQLITE_OK) on success, or an SQLite error code on failure.
    ///
    /// # Safety
    ///
    /// `app_data` may be null or point to application-specific data.
    /// This method must be safe to call exactly once per allocator instance.
    unsafe fn init(&mut self, app_data: *mut c_void) -> c_int;

    /// Shutdown the memory allocator.
    ///
    /// This is called once when SQLite shuts down.
    /// All allocated memory should be freed before this is called.
    ///
    /// # Safety
    ///
    /// `app_data` may be null or point to application-specific data.
    /// This method must be safe to call exactly once per allocator instance.
    unsafe fn shutdown(&mut self, app_data: *mut c_void);
}

/// Global storage for the configured memory allocator.
/// Uses `OnceLock` to ensure it can only be set once.
static ALLOCATOR: OnceLock<Mutex<Box<dyn SqliteMemoryAllocator>>> = OnceLock::new();

/// Configure SQLite to use a custom memory allocator.
///
/// This function must be called before creating any SQLite database connections.
/// It can only be called once per process - subsequent calls will return an error.
///
/// # Arguments
///
/// * `allocator` - The custom allocator implementation
///
/// # Returns
///
/// * `Ok(())` - If the allocator was successfully configured
/// * `Err(sqlx_core::Error)` - If configuration failed or was called after connections were created
///
/// # Example
///
/// ```rust,no_run
/// use sqlx_sqlite::memory::{SqliteMemoryAllocator, configure_memory_allocator};
///
/// struct MyAllocator;
///
/// unsafe impl SqliteMemoryAllocator for MyAllocator {
///     // ... implement all required methods
/// #    unsafe fn malloc(&mut self, size: std::os::raw::c_int) -> *mut std::ffi::c_void { std::ptr::null_mut() }
/// #    unsafe fn free(&mut self, ptr: *mut std::ffi::c_void) {}
/// #    unsafe fn realloc(&mut self, ptr: *mut std::ffi::c_void, size: std::os::raw::c_int) -> *mut std::ffi::c_void { std::ptr::null_mut() }
/// #    unsafe fn size(&mut self, ptr: *mut std::ffi::c_void) -> std::os::raw::c_int { 0 }
/// #    unsafe fn roundup(&mut self, size: std::os::raw::c_int) -> std::os::raw::c_int { size }
/// #    unsafe fn init(&mut self, app_data: *mut std::ffi::c_void) -> std::os::raw::c_int { 0 }
/// #    unsafe fn shutdown(&mut self, app_data: *mut std::ffi::c_void) {}
/// }
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// configure_memory_allocator(MyAllocator)?;
/// # Ok(())
/// # }
/// ```
pub fn configure_memory_allocator<A>(allocator: A) -> Result<(), sqlx_core::Error>
where
    A: SqliteMemoryAllocator,
{
    let boxed_allocator: Box<dyn SqliteMemoryAllocator> = Box::new(allocator);
    let mutex_allocator = Mutex::new(boxed_allocator);

    // Try to set the allocator - this will fail if already set
    ALLOCATOR.set(mutex_allocator).map_err(|_| {
        sqlx_core::Error::Configuration(
            "Memory allocator has already been configured. \
            configure_memory_allocator() can only be called once per process."
                .into(),
        )
    })?;

    // Create the sqlite3_mem_methods structure
    let mem_methods = sqlite3_mem_methods {
        xMalloc: Some(malloc_wrapper),
        xFree: Some(free_wrapper),
        xRealloc: Some(realloc_wrapper),
        xSize: Some(size_wrapper),
        xRoundup: Some(roundup_wrapper),
        xInit: Some(init_wrapper),
        xShutdown: Some(shutdown_wrapper),
        pAppData: ptr::null_mut(),
    };

    // Configure SQLite to use our custom allocator
    let result = unsafe { sqlite3_config(SQLITE_CONFIG_MALLOC, &mem_methods) };

    if result != SQLITE_OK {
        return Err(sqlx_core::Error::Configuration(
            format!(
                "Failed to configure SQLite memory allocator: error code {}",
                result
            )
            .into(),
        ));
    }

    Ok(())
}

// C wrapper functions that call into our Rust allocator

extern "C" fn malloc_wrapper(size: c_int) -> *mut c_void {
    let allocator = match ALLOCATOR.get() {
        Some(alloc) => alloc,
        None => return ptr::null_mut(),
    };

    let mut guard = match allocator.lock() {
        Ok(guard) => guard,
        Err(_) => return ptr::null_mut(),
    };

    unsafe { guard.malloc(size) }
}

extern "C" fn free_wrapper(ptr: *mut c_void) {
    let allocator = match ALLOCATOR.get() {
        Some(alloc) => alloc,
        None => return,
    };

    let mut guard = match allocator.lock() {
        Ok(guard) => guard,
        Err(_) => return,
    };

    unsafe { guard.free(ptr) }
}

extern "C" fn realloc_wrapper(ptr: *mut c_void, size: c_int) -> *mut c_void {
    let allocator = match ALLOCATOR.get() {
        Some(alloc) => alloc,
        None => return ptr::null_mut(),
    };

    let mut guard = match allocator.lock() {
        Ok(guard) => guard,
        Err(_) => return ptr::null_mut(),
    };

    unsafe { guard.realloc(ptr, size) }
}

extern "C" fn size_wrapper(ptr: *mut c_void) -> c_int {
    let allocator = match ALLOCATOR.get() {
        Some(alloc) => alloc,
        None => return 0,
    };

    let mut guard = match allocator.lock() {
        Ok(guard) => guard,
        Err(_) => return 0,
    };

    unsafe { guard.size(ptr) }
}

extern "C" fn roundup_wrapper(size: c_int) -> c_int {
    let allocator = match ALLOCATOR.get() {
        Some(alloc) => alloc,
        None => return size,
    };

    let mut guard = match allocator.lock() {
        Ok(guard) => guard,
        Err(_) => return size,
    };

    unsafe { guard.roundup(size) }
}

extern "C" fn init_wrapper(_app_data: *mut c_void) -> c_int {
    let allocator = match ALLOCATOR.get() {
        Some(alloc) => alloc,
        None => return 1, // SQLITE_ERROR
    };

    let mut guard = match allocator.lock() {
        Ok(guard) => guard,
        Err(_) => return 1, // SQLITE_ERROR
    };

    unsafe { guard.init(_app_data) }
}

extern "C" fn shutdown_wrapper(app_data: *mut c_void) {
    let allocator = match ALLOCATOR.get() {
        Some(alloc) => alloc,
        None => return,
    };

    let mut guard = match allocator.lock() {
        Ok(guard) => guard,
        Err(_) => return,
    };

    unsafe { guard.shutdown(app_data) }
}
