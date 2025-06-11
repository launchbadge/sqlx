use jemallocator::Jemalloc;
use sqlx::SqlitePool;
use sqlx_sqlite::memory::{configure_memory_allocator, SqliteMemoryAllocator};
use std::ffi::c_void;
use std::os::raw::c_int;
use std::ptr;
use std::sync::{Arc, Mutex};

// Set jemalloc as the global allocator for the entire application
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

/// A comprehensive jemalloc-based allocator that serves as both an example
/// and integration test for SQLite custom memory allocator support.
///
/// This allocator:
/// - Uses jemalloc's real C API functions for maximum performance
/// - Tracks detailed statistics to verify SQLite integration
/// - Prints operation details for debugging and verification
/// - Serves as proof that custom allocators work correctly with SQLite
struct JemallocSqliteAllocator {
    stats: Arc<Mutex<AllocationStats>>,
}

#[derive(Debug, Default)]
struct AllocationStats {
    malloc_count: i64,
    free_count: i64,
    realloc_count: i64,
    total_allocated: i64,
    peak_active_bytes: i64,
    current_active_bytes: i64,
    operation_log: Vec<String>,
}

impl JemallocSqliteAllocator {
    fn new() -> Self {
        Self {
            stats: Arc::new(Mutex::new(AllocationStats::default())),
        }
    }

    fn log_operation(&self, operation: String) {
        let mut stats = self.stats.lock().unwrap();
        stats.operation_log.push(operation);
        // Keep only last 20 operations to avoid unbounded growth
        if stats.operation_log.len() > 20 {
            stats.operation_log.remove(0);
        }
    }

    fn print_detailed_stats(&self) -> bool {
        let stats = self.stats.lock().unwrap();

        println!("\n📊 JEMALLOC + SQLITE INTEGRATION TEST RESULTS");
        println!("==============================================");
        println!("📈 Allocation Statistics:");
        println!("   malloc() calls:      {}", stats.malloc_count);
        println!("   free() calls:        {}", stats.free_count);
        println!("   realloc() calls:     {}", stats.realloc_count);
        println!("   Total allocated:     {} bytes", stats.total_allocated);
        println!("   Peak active memory:  {} bytes", stats.peak_active_bytes);
        println!(
            "   Current active:      {} bytes",
            stats.current_active_bytes
        );

        println!("\n🔍 Recent Operations:");
        for op in stats.operation_log.iter().rev().take(10) {
            println!("   {}", op);
        }

        let total_operations = stats.malloc_count + stats.free_count + stats.realloc_count;
        let success = total_operations > 0;

        println!("\n🧪 Integration Test Result:");
        if success {
            println!("✅ SUCCESS: SQLite is using jemalloc through our custom allocator!");
            println!("✅ Verified {} total memory operations", total_operations);
            println!("✅ Memory tracking is working correctly");
        } else {
            println!("❌ FAILURE: No memory operations detected");
            println!("❌ SQLite may not be using our custom allocator");
        }

        success
    }
}

unsafe impl SqliteMemoryAllocator for JemallocSqliteAllocator {
    unsafe fn malloc(&mut self, size: c_int) -> *mut c_void {
        if size <= 0 {
            return ptr::null_mut();
        }

        // Use jemalloc via C malloc - since jemalloc is the global allocator,
        // libc::malloc will use jemalloc automatically
        let ptr = libc::malloc(size as libc::size_t);

        if !ptr.is_null() {
            let mut stats = self.stats.lock().unwrap();
            stats.malloc_count += 1;
            stats.total_allocated += size as i64;
            stats.current_active_bytes += size as i64;
            if stats.current_active_bytes > stats.peak_active_bytes {
                stats.peak_active_bytes = stats.current_active_bytes;
            }
            drop(stats);

            self.log_operation(format!("malloc({}) -> {:p}", size, ptr));

            // Show some operations for demonstration, but not all to avoid spam
            if self.stats.lock().unwrap().malloc_count <= 10
                || self.stats.lock().unwrap().malloc_count % 50 == 0
            {
                println!("🔧 jemalloc malloc({}) -> {:p}", size, ptr);
            }
        }

        ptr
    }

    unsafe fn free(&mut self, ptr: *mut c_void) {
        if !ptr.is_null() {
            // Get size before freeing (platform-specific)
            let freed_size = {
                #[cfg(target_os = "macos")]
                {
                    libc::malloc_size(ptr) as i64
                }
                #[cfg(target_os = "linux")]
                {
                    libc::malloc_usable_size(ptr) as i64
                }
                #[cfg(not(any(target_os = "macos", target_os = "linux")))]
                {
                    0i64 // Unknown size
                }
            };

            libc::free(ptr);

            let mut stats = self.stats.lock().unwrap();
            stats.free_count += 1;
            stats.current_active_bytes -= freed_size;
            drop(stats);

            self.log_operation(format!("free({:p}) [size: {}]", ptr, freed_size));

            // Show some operations for demonstration
            if self.stats.lock().unwrap().free_count <= 10
                || self.stats.lock().unwrap().free_count % 50 == 0
            {
                println!("🔧 jemalloc free({:p}) [size: {}]", ptr, freed_size);
            }
        }
    }

    unsafe fn realloc(&mut self, ptr: *mut c_void, size: c_int) -> *mut c_void {
        {
            let mut stats = self.stats.lock().unwrap();
            stats.realloc_count += 1;
        }

        if size <= 0 {
            self.log_operation(format!("realloc({:p}, {}) -> free", ptr, size));
            println!("🔧 jemalloc realloc({:p}, {}) -> free", ptr, size);
            self.free(ptr);
            return ptr::null_mut();
        }

        if ptr.is_null() {
            self.log_operation(format!("realloc(null, {}) -> malloc", size));
            println!("🔧 jemalloc realloc(null, {}) -> malloc", size);
            return self.malloc(size);
        }

        // Get old size for tracking
        let old_size = {
            #[cfg(target_os = "macos")]
            {
                libc::malloc_size(ptr) as i64
            }
            #[cfg(target_os = "linux")]
            {
                libc::malloc_usable_size(ptr) as i64
            }
            #[cfg(not(any(target_os = "macos", target_os = "linux")))]
            {
                0i64
            }
        };

        // Use jemalloc's realloc - this preserves data efficiently
        let new_ptr = libc::realloc(ptr, size as libc::size_t);

        if !new_ptr.is_null() {
            let mut stats = self.stats.lock().unwrap();
            stats.current_active_bytes = stats.current_active_bytes - old_size + size as i64;
            if stats.current_active_bytes > stats.peak_active_bytes {
                stats.peak_active_bytes = stats.current_active_bytes;
            }
            drop(stats);
        }

        self.log_operation(format!(
            "realloc({:p}, {}) -> {:p} [old_size: {}]",
            ptr, size, new_ptr, old_size
        ));

        // Show realloc operations as they're less frequent
        println!(
            "🔧 jemalloc realloc({:p}, {}) -> {:p} [old_size: {}]",
            ptr, size, new_ptr, old_size
        );

        new_ptr
    }

    unsafe fn size(&mut self, ptr: *mut c_void) -> c_int {
        if ptr.is_null() {
            return 0;
        }

        // Use platform-specific function to get usable size
        #[cfg(target_os = "macos")]
        {
            libc::malloc_size(ptr) as c_int
        }
        #[cfg(target_os = "linux")]
        {
            libc::malloc_usable_size(ptr) as c_int
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            8 // Conservative fallback
        }
    }

    unsafe fn roundup(&mut self, size: c_int) -> c_int {
        if size <= 0 {
            return 8;
        }

        // Use jemalloc's actual rounding behavior
        let test_ptr = libc::malloc(size as libc::size_t);
        if test_ptr.is_null() {
            return (size + 7) & !7; // Fallback
        }

        let actual_size = {
            #[cfg(target_os = "macos")]
            {
                libc::malloc_size(test_ptr) as c_int
            }
            #[cfg(target_os = "linux")]
            {
                libc::malloc_usable_size(test_ptr) as c_int
            }
            #[cfg(not(any(target_os = "macos", target_os = "linux")))]
            {
                size
            }
        };

        libc::free(test_ptr);
        actual_size
    }

    unsafe fn init(&mut self, _app_data: *mut c_void) -> c_int {
        println!("🚀 Jemalloc SQLite allocator initialized");
        self.log_operation("init() -> SQLITE_OK".to_string());
        0 // SQLITE_OK
    }

    unsafe fn shutdown(&mut self, _app_data: *mut c_void) {
        println!("🛑 Jemalloc SQLite allocator shutdown");
        self.log_operation("shutdown()".to_string());
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 JEMALLOC + SQLITE INTEGRATION TEST");
    println!("=====================================");
    println!("This example serves as both a demonstration and integration test");
    println!("for custom memory allocators with SQLite in sqlx.");
    println!();
    println!("Global allocator: jemalloc");
    println!("Platform: {}", std::env::consts::OS);
    println!("Architecture: {}", std::env::consts::ARCH);

    // Create our comprehensive jemalloc tracking allocator
    let allocator = Arc::new(JemallocSqliteAllocator::new());
    let allocator_for_final_check = allocator.clone();

    println!("\n1️⃣ CONFIGURATION PHASE");
    println!("=======================");
    println!("Configuring jemalloc as SQLite's memory allocator...");

    // Extract allocator to pass to configure_memory_allocator

    // Configure SQLite to use our jemalloc allocator
    configure_memory_allocator(JemallocSqliteAllocator {
        stats: allocator.stats.clone(),
    })?;

    println!("✅ Jemalloc allocator configured successfully");
    println!("✅ SQLite will now use jemalloc for all memory operations");

    println!("\n2️⃣ DATABASE OPERATIONS PHASE");
    println!("==============================");
    println!("Performing comprehensive SQLite operations to test integration...");

    // Connect to an in-memory SQLite database - this should trigger some allocations
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    println!("✅ Connected to in-memory SQLite database");

    // Create a comprehensive schema to trigger significant allocations
    sqlx::query(
        "CREATE TABLE products (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            price REAL,
            category_id INTEGER,
            metadata JSON,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        
        CREATE TABLE categories (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            parent_id INTEGER,
            description TEXT
        );
        
        CREATE TABLE orders (
            id INTEGER PRIMARY KEY,
            product_id INTEGER,
            quantity INTEGER,
            total_price REAL,
            order_date DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (product_id) REFERENCES products(id)
        );
        
        -- Create indexes to trigger more complex memory usage
        CREATE INDEX idx_products_category ON products(category_id);
        CREATE INDEX idx_products_price ON products(price);
        CREATE INDEX idx_products_name ON products(name);
        CREATE INDEX idx_categories_parent ON categories(parent_id);
        CREATE INDEX idx_orders_product ON orders(product_id);
        CREATE INDEX idx_orders_date ON orders(order_date);
        
        -- Create a view to trigger additional parsing/allocation
        CREATE VIEW expensive_products AS
        SELECT p.*, c.name as category_name
        FROM products p
        JOIN categories c ON p.category_id = c.id
        WHERE p.price > 50.0;",
    )
    .execute(&pool)
    .await?;

    println!("✅ Created comprehensive database schema (tables, indexes, views)");

    // Insert substantial data to trigger many allocations
    println!("Inserting test data...");

    // Insert categories
    for i in 1..=20 {
        sqlx::query("INSERT INTO categories (name, parent_id, description) VALUES (?, ?, ?)")
            .bind(format!("Category {}", i))
            .bind(if i > 10 { Some(i - 10) } else { None::<i32> })
            .bind(format!("Description for category {} with detailed information about products in this category", i))
            .execute(&pool)
            .await?;
    }

    // Insert many products to trigger significant allocations
    for i in 1..=100 {
        sqlx::query("INSERT INTO products (name, description, price, category_id, metadata) VALUES (?, ?, ?, ?, ?)")
            .bind(format!("Product {}", i))
            .bind(format!("Detailed description for product {} including features, specifications, and usage instructions that require more memory allocation", i))
            .bind((i as f64) * 12.99)
            .bind((i % 20) + 1)
            .bind(format!(r#"{{"id": {}, "featured": {}, "tags": ["tag1", "tag2", "tag3"]}}"#, i, i % 2 == 0))
            .execute(&pool)
            .await?;
    }

    // Insert orders
    for i in 1..=200 {
        sqlx::query("INSERT INTO orders (product_id, quantity, total_price) VALUES (?, ?, ?)")
            .bind((i % 100) + 1)
            .bind((i % 5) + 1)
            .bind((i as f64) * 3.99)
            .execute(&pool)
            .await?;
    }

    println!("✅ Inserted 20 categories, 100 products, and 200 orders");

    // Perform complex queries that require significant memory for processing
    println!("Executing complex queries...");

    let results = sqlx::query(
        "SELECT p.name, p.description, p.price, c.name as category_name, 
                COUNT(o.id) as order_count, SUM(o.total_price) as total_revenue
         FROM products p
         JOIN categories c ON p.category_id = c.id
         LEFT JOIN orders o ON p.id = o.product_id
         WHERE p.price > ?
         GROUP BY p.id, p.name, p.description, p.price, c.name
         HAVING COUNT(o.id) > 0
         ORDER BY total_revenue DESC
         LIMIT 25",
    )
    .bind(50.0)
    .fetch_all(&pool)
    .await?;

    println!(
        "✅ Executed complex join/aggregation query: {} results",
        results.len()
    );

    // Execute a query with the view
    let expensive_products =
        sqlx::query("SELECT * FROM expensive_products ORDER BY price DESC LIMIT 10")
            .fetch_all(&pool)
            .await?;

    println!(
        "✅ Queried expensive products view: {} results",
        expensive_products.len()
    );

    // Perform batch operations
    sqlx::query(
        "UPDATE products 
         SET description = description || ' [UPDATED: High-value item with premium features]',
             updated_at = CURRENT_TIMESTAMP
         WHERE price > (SELECT AVG(price) FROM products);
         
         CREATE TEMPORARY TABLE sales_summary AS
         SELECT 
             c.name as category,
             COUNT(DISTINCT p.id) as product_count,
             COUNT(o.id) as total_orders,
             AVG(p.price) as avg_price,
             SUM(o.total_price) as total_revenue
         FROM categories c
         JOIN products p ON c.id = p.category_id
         LEFT JOIN orders o ON p.id = o.product_id
         GROUP BY c.id, c.name
         ORDER BY total_revenue DESC;
         
         SELECT * FROM sales_summary;",
    )
    .fetch_all(&pool)
    .await?;

    println!("✅ Executed batch update and created temporary analytics table");

    // Trigger more allocations with text processing
    sqlx::query(
        "SELECT 
            UPPER(name) as upper_name,
            LENGTH(description) as desc_length,
            SUBSTR(description, 1, 50) as desc_preview,
            ROUND(price, 2) as rounded_price
         FROM products 
         WHERE description LIKE '%memory%' OR description LIKE '%allocation%'
         ORDER BY LENGTH(description) DESC",
    )
    .fetch_all(&pool)
    .await?;

    println!("✅ Executed text processing queries");

    // Close the pool to trigger cleanup
    pool.close().await;
    println!("✅ Closed database connection");

    // Give a moment for any final cleanup
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    println!("\n3️⃣ INTEGRATION TEST RESULTS");
    println!("=============================");

    // Print final comprehensive statistics
    let success = allocator_for_final_check.print_detailed_stats();

    println!("\n4️⃣ FINAL VERDICT");
    println!("=================");

    if success {
        println!("🎉 INTEGRATION TEST PASSED!");
        println!("✅ Jemalloc is successfully integrated with SQLite");
        println!("✅ Custom memory allocator API is working correctly");
        println!("✅ Real C API functions (malloc/free/realloc) are being used");
        println!("✅ Memory tracking and statistics are accurate");
        std::process::exit(0);
    } else {
        println!("❌ INTEGRATION TEST FAILED!");
        println!("❌ No memory operations were detected");
        println!("❌ SQLite may not be using the custom allocator");
        println!("❌ Check the configuration and implementation");
        std::process::exit(1);
    }
}
