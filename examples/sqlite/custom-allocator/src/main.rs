use sqlx::{Row, SqlitePool};
use sqlx_sqlite::memory::{configure_memory_allocator, SqliteMemoryAllocator};
use std::alloc::{GlobalAlloc, Layout, System};
use std::collections::HashMap;
use std::ffi::c_void;
use std::os::raw::c_int;
use std::ptr;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

/// A tracking allocator that records all SQLite memory operations
/// and allows us to verify that SQLite is actually using our custom allocator.
struct TrackingAllocator {
    // Track allocations by converting pointer to usize for HashMap key
    allocations: Arc<Mutex<HashMap<usize, usize>>>,
    malloc_count: Arc<AtomicI64>,
    free_count: Arc<AtomicI64>,
    realloc_count: Arc<AtomicI64>,
    total_allocated: Arc<AtomicI64>,
}

impl TrackingAllocator {
    fn new() -> Self {
        Self {
            allocations: Arc::new(Mutex::new(HashMap::new())),
            malloc_count: Arc::new(AtomicI64::new(0)),
            free_count: Arc::new(AtomicI64::new(0)),
            realloc_count: Arc::new(AtomicI64::new(0)),
            total_allocated: Arc::new(AtomicI64::new(0)),
        }
    }
}

unsafe impl SqliteMemoryAllocator for TrackingAllocator {
    unsafe fn malloc(&mut self, size: c_int) -> *mut c_void {
        if size <= 0 {
            return ptr::null_mut();
        }

        let layout = match Layout::from_size_align(size as usize, 8) {
            Ok(layout) => layout,
            Err(_) => return ptr::null_mut(),
        };

        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            self.allocations
                .lock()
                .unwrap()
                .insert(ptr as usize, size as usize);
            self.malloc_count.fetch_add(1, Ordering::Relaxed);
            self.total_allocated
                .fetch_add(size as i64, Ordering::Relaxed);

            println!("🔧 malloc({}) -> {:p}", size, ptr);
        }

        ptr as *mut c_void
    }

    unsafe fn free(&mut self, ptr: *mut c_void) {
        if !ptr.is_null() {
            if let Some(size) = self.allocations.lock().unwrap().remove(&(ptr as usize)) {
                let layout = Layout::from_size_align_unchecked(size, 8);
                System.dealloc(ptr as *mut u8, layout);
                self.free_count.fetch_add(1, Ordering::Relaxed);

                println!("🔧 free({:p}) [size: {}]", ptr, size);
            }
        }
    }

    unsafe fn realloc(&mut self, ptr: *mut c_void, size: c_int) -> *mut c_void {
        self.realloc_count.fetch_add(1, Ordering::Relaxed);

        if size <= 0 {
            println!("🔧 realloc({:p}, {}) -> free", ptr, size);
            self.free(ptr);
            return ptr::null_mut();
        }

        if ptr.is_null() {
            println!("🔧 realloc(null, {}) -> malloc", size);
            return self.malloc(size);
        }

        // Get old size for copying data
        let old_size = self
            .allocations
            .lock()
            .unwrap()
            .get(&(ptr as usize))
            .copied()
            .unwrap_or(0);

        // Allocate new memory
        let new_ptr = self.malloc(size);
        if !new_ptr.is_null() && !ptr.is_null() {
            // Copy old data to new location
            let copy_size = std::cmp::min(old_size, size as usize);
            ptr::copy_nonoverlapping(ptr as *const u8, new_ptr as *mut u8, copy_size);
            self.free(ptr);

            println!(
                "🔧 realloc({:p}, {}) -> {:p} [copied {} bytes]",
                ptr, size, new_ptr, copy_size
            );
        }

        new_ptr
    }

    unsafe fn size(&mut self, ptr: *mut c_void) -> c_int {
        if ptr.is_null() {
            return 0;
        }

        self.allocations
            .lock()
            .unwrap()
            .get(&(ptr as usize))
            .copied()
            .unwrap_or(0) as c_int
    }

    unsafe fn roundup(&mut self, size: c_int) -> c_int {
        // Round up to next multiple of 8
        (size + 7) & !7
    }

    unsafe fn init(&mut self, _app_data: *mut c_void) -> c_int {
        println!("🚀 SQLite memory allocator initialized");
        0 // SQLITE_OK
    }

    unsafe fn shutdown(&mut self, _app_data: *mut c_void) {
        println!("🛑 SQLite memory allocator shutdown");
        self.allocations.lock().unwrap().clear();
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 SQLite Custom Memory Allocator Example");
    println!("==========================================");

    // Create our tracking allocator
    let tracking_allocator = TrackingAllocator::new();

    println!("\n1️⃣ Configuring custom memory allocator...");

    // Configure SQLite to use our custom allocator
    // This MUST be done before any SQLite operations
    configure_memory_allocator(tracking_allocator)?;

    println!("✅ Custom allocator configured successfully");

    println!("\n2️⃣ Performing SQLite operations...");

    // Connect to an in-memory SQLite database
    let pool = SqlitePool::connect("sqlite::memory:").await?;

    // Create a table
    sqlx::query(
        "CREATE TABLE users (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    println!("✅ Created table");

    // Insert some data
    for i in 1..=5 {
        sqlx::query("INSERT INTO users (name, email) VALUES (?, ?)")
            .bind(format!("User {}", i))
            .bind(format!("user{}@example.com", i))
            .execute(&pool)
            .await?;
    }

    println!("✅ Inserted 5 users");

    // Query the data
    let rows = sqlx::query("SELECT id, name, email FROM users ORDER BY id")
        .fetch_all(&pool)
        .await?;

    println!("✅ Queried {} rows:", rows.len());
    for row in rows {
        let id: i64 = row.get("id");
        let name: String = row.get("name");
        let email: String = row.get("email");
        println!("   {} - {} ({})", id, name, email);
    }

    // Perform some more complex operations to trigger more allocations
    sqlx::query(
        "CREATE INDEX idx_users_email ON users(email);
         CREATE VIEW user_summary AS SELECT COUNT(*) as total FROM users;
         SELECT * FROM user_summary;",
    )
    .fetch_all(&pool)
    .await?;

    println!("✅ Created index and view");

    // Close the pool to trigger cleanup
    pool.close().await;

    println!("\n3️⃣ Memory allocator statistics:");
    println!("================================");

    // Note: We can't directly access the allocator instance after it's been
    // passed to configure_memory_allocator(), but the allocator will have
    // printed statistics during the operations above.

    println!("\n🎉 Example completed successfully!");
    println!("\nIf you see malloc/free/realloc calls above, it means SQLite");
    println!("is successfully using our custom memory allocator!");

    Ok(())
}
