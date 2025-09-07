# RSB Adapter Pattern for Complex Integrations

**Context**: Sometimes RSB needs to integrate with complex systems (databases, crypto, networking) that are better served with standard Rust patterns rather than string-first approaches.

**Solution**: Use adapter/strategy pattern to create isolated modules with standard Rust power, then wire them back into RSB architecture through clean string-first interfaces.

## Core Principles

### ✅ **When to Use Adapters**
- Database integration (SQLite, PostgreSQL) with complex queries
- Cryptographic operations requiring type safety
- Network protocols with structured data
- Performance-critical operations
- Third-party libraries with their own patterns

### ❌ **When NOT to Use Adapters** 
- Simple file operations (use RSB patterns)
- Basic string processing (native RSB strength)
- Command-line argument parsing (RSB handles this)
- Configuration management (RSB's specialty)

## Implementation Pattern

### 1. **Isolated Standard Rust Module**
```rust
// src/prontodb/adapters/database.rs
// This module can use full Rust power - types, traits, etc.

use rusqlite::{Connection, Result, Row};
use serde_json::Value;

pub struct DatabaseAdapter {
    conn: Connection,
}

impl DatabaseAdapter {
    pub fn new(db_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let conn = Connection::open(db_path)?;
        Ok(Self { conn })
    }
    
    // Standard Rust patterns - complex types OK here
    pub fn insert_key_value(&self, project: &str, namespace: &str, key: &str, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
        // Full SQL power, proper error handling, etc.
    }
    
    pub fn get_value(&self, project: &str, namespace: &str, key: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        // Complex query logic with proper typing
    }
}
```

### 2. **RSB String-First Interface**
```rust
// src/prontodb/core.rs - Business logic tier
use super::adapters::database::DatabaseAdapter;

// String-first wrapper around the adapter
pub fn _helper_set(key: &str, value: &str) -> Result<(), String> {
    let addr = _parse_address(key)?;
    
    // Initialize adapter (could be cached/singleton)
    let db = _get_database_adapter()?;
    
    // Convert RSB strings to adapter format
    let json_value: serde_json::Value = serde_json::from_str(value)
        .unwrap_or_else(|_| serde_json::Value::String(value.to_string()));
    
    // Call adapter with string parameters
    db.insert_key_value(&addr.project, &addr.namespace, &addr.key, &json_value)
        .map_err(|e| e.to_string())?;
        
    Ok(())
}

pub fn _helper_get(key: &str) -> Result<Option<String>, String> {
    let addr = _parse_address(key)?;
    let db = _get_database_adapter()?;
    
    // Adapter returns standard Rust Result, we convert to RSB string result
    db.get_value(&addr.project, &addr.namespace, &addr.key)
        .map_err(|e| e.to_string())
}

// RSB pattern: string-first configuration
fn _get_database_adapter() -> Result<DatabaseAdapter, String> {
    let home = std::env::var("HOME").map_err(|_| "HOME not set")?;
    let db_path = format!("{}/.local/data/odx/prontodb/store.db", home);
    
    DatabaseAdapter::new(&db_path).map_err(|e| e.to_string())
}
```

### 3. **Module Structure**
```
src/prontodb/
├── mod.rs              # Re-exports
├── handlers.rs         # RSB public API (do_* functions)
├── core.rs            # RSB business logic (_helper_* functions)  
├── utils.rs           # RSB system operations (__blind_faith_* functions)
└── adapters/          # Standard Rust modules
    ├── mod.rs         # Adapter interface
    ├── database.rs    # SQLite adapter
    ├── crypto.rs      # Encryption adapter (optional)
    └── network.rs     # HTTP adapter (optional)
```

### 4. **Clean Module Interface**
```rust
// src/prontodb/adapters/mod.rs
// Optional: Define common traits for adapters

pub mod database;

// Common adapter patterns could go here
pub trait StorageAdapter {
    fn set(&self, key: &str, value: &str) -> Result<(), String>;
    fn get(&self, key: &str) -> Result<Option<String>, String>;
}
```

## Key Benefits

### 🏗️ **Architectural Integrity**
- RSB string-first interface preserved
- Complex Rust power isolated in adapters
- Clean separation of concerns
- Easy to test both layers independently

### 🔄 **Flexibility** 
- Can swap adapters (SQLite → PostgreSQL)
- Standard Rust ecosystem compatibility  
- Performance optimization possible in adapters
- Type safety where it matters

### 📦 **RSB Compliance**
- Three-tier function ordinality maintained
- String-first public interfaces
- Error handling converted to RSB patterns
- Configuration stays simple

## Implementation Strategy for ProntoDB

### Phase 1: Basic SQLite Adapter
```rust
// Start with simple key-value operations
DatabaseAdapter::set_kv(project, namespace, key, value) -> Result<(), Error>
DatabaseAdapter::get_kv(project, namespace, key) -> Result<Option<String>, Error>
DatabaseAdapter::del_kv(project, namespace, key) -> Result<usize, Error>
```

### Phase 2: Advanced Features
```rust
// Add TTL, JSON queries, transactions
DatabaseAdapter::set_with_ttl(...)
DatabaseAdapter::query_json(...)  
DatabaseAdapter::transaction(...)
```

### Phase 3: Multi-Backend Support
```rust
// Strategy pattern for different databases
trait StorageBackend { ... }
impl StorageBackend for SqliteBackend { ... }
impl StorageBackend for PostgresBackend { ... }
```

## Critical Requirements

### ⚠️ **RSB Interface Contract**
- **Input**: Always strings from RSB layer
- **Output**: Always strings or simple errors back to RSB
- **Configuration**: String-based paths, simple parameters
- **No RSB Pollution**: Adapters should not know about RSB patterns

### 🔐 **Error Translation**
```rust
// Adapter complex errors → RSB string errors
adapter_result.map_err(|complex_error| {
    // Log detailed error internally
    log::error!("Database error: {:?}", complex_error);
    // Return simple string for RSB
    "Database operation failed".to_string()
})
```

This adapter pattern gives us **"escape hatch"** power when needed while preserving RSB's elegant simplicity for users and most of the codebase.

---
*RSB Adapter Pattern Documentation*  
*Bridging String-First Simplicity with Rust Power*