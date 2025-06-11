# SQLite Custom Memory Allocator Examples

This directory contains examples demonstrating how to use custom memory allocators with SQLite in sqlx.

## Examples

### 1. Basic Tracking Allocator (`basic`)

A simple example that wraps the system allocator and tracks all SQLite memory operations.

**Run:**
```bash
cargo run --bin basic
```

**Features:**
- Tracks malloc/free/realloc calls
- Shows real SQLite memory usage patterns
- Demonstrates basic allocator configuration

### 2. Jemalloc Integration (`jemalloc`)

A high-performance example using jemalloc as both the global allocator and SQLite's memory allocator.

**Run:**
```bash
cargo run --bin jemalloc
```

**Features:**
- Uses jemalloc's real C API functions
- Shows platform-specific memory size functions
- Demonstrates jemalloc's allocation rounding behavior
- Performs complex SQLite operations to trigger more allocations

## Key Concepts Demonstrated

### Memory Allocator Configuration
```rust
// MUST be called before any SQLite operations
configure_memory_allocator(my_allocator)?;

// Now all SQLite operations use your custom allocator
let pool = SqlitePool::connect("sqlite::memory:").await?;
```

### Global Allocator Setup (for jemalloc)
```rust
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;
```

### Verification
Both examples print detailed information about:
- Number of malloc/free/realloc calls
- Total memory allocated
- Individual allocation details
- Whether SQLite is actually using the custom allocator

## Expected Output

When you run the examples, you should see output like:
```
🔧 malloc(1024) -> 0x7f8b4c000800
🔧 malloc(256) -> 0x7f8b4c000900
🔧 realloc(0x7f8b4c000800, 2048) -> 0x7f8b4c001000 [copied 1024 bytes]
🔧 free(0x7f8b4c000900) [size: 256]

📊 SQLite Memory Allocator Statistics:
   malloc() calls:     45
   free() calls:       23
   realloc() calls:    8
   Total allocated:    15,360 bytes
   Active allocations: 22

✅ SUCCESS: SQLite is using our custom allocator!
```

## Use Cases

These examples are useful for:
- **Memory profiling**: Track SQLite's memory usage patterns
- **Performance optimization**: Use high-performance allocators like jemalloc
- **Memory debugging**: Detect leaks or excessive allocations
- **Resource constraints**: Implement custom allocation limits
- **Testing**: Simulate out-of-memory conditions

## Important Notes

1. **Configuration timing**: `configure_memory_allocator()` must be called before any SQLite operations
2. **One-time setup**: Can only be configured once per process
3. **Global effect**: Affects all SQLite databases in the process
4. **Thread safety**: The allocator wrapper handles thread synchronization

## Building and Running

From this directory:
```bash
# Basic example
cargo run --bin basic

# Jemalloc example  
cargo run --bin jemalloc

# Run with more verbose output
RUST_LOG=debug cargo run --bin basic
```

The examples will create in-memory SQLite databases and perform various operations to demonstrate the allocator integration.