// src/db.rs

// This module provides a simple, string-based interface for database
// operations, defaulting to SQLite.
use rusqlite::{Connection, Result};
use std::collections::HashMap;

// A simple connection manager. In a real app, this would be more robust.
fn open_db() -> Result<Connection> {
    // The database path can be configured via an environment variable.
    let db_path = crate::context::get_var("DB_URL");
    Connection::open(if db_path.is_empty() { "rsb_db.sqlite" } else { &db_path })
}

/// Executes a query that is expected to return rows.
pub fn db_query(query: &str, params: &[&str]) -> Result<Vec<HashMap<String, String>>> {
    let conn = open_db()?;
    let mut stmt = conn.prepare(query)?;

    let col_names: Vec<String> = stmt.column_names().into_iter().map(|s| s.to_string()).collect();

    let mut rows = stmt.query(rusqlite::params_from_iter(params))?;

    let mut results = Vec::new();
    while let Some(row) = rows.next()? {
        let mut map = HashMap::new();
        for (i, col_name) in col_names.iter().enumerate() {
            let val: String = row.get(i)?;
            map.insert(col_name.clone(), val);
        }
        results.push(map);
    }
    Ok(results)
}

/// Executes a statement that does not return rows (INSERT, UPDATE, DELETE).
pub fn db_exec(statement: &str, params: &[&str]) -> Result<usize> {
    let conn = open_db()?;
    conn.execute(statement, rusqlite::params_from_iter(params))
}
