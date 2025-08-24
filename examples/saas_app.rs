//! RSB SaaS Application Example
//!
//! This example demonstrates building a simple micro-SaaS application using RSB.
//! It sets up a web server with a few API endpoints for user management,
//! including registration, login (issuing a JWT), and a protected resource.
//!
//! ## Prerequisites
//!
//! Before running, make sure you have a `JWT_SECRET` environment variable set.
//! You can set one for testing purposes like this:
//!
//! ```sh
//! export JWT_SECRET="your-super-secret-and-long-key-here"
//! ```
//!
//! ## Running the Example
//!
//! ```sh
//! cargo run --example saas_app
//! ```
//!
//! ## API Endpoints
//!
//! - `POST /register`: Register a new user.
//!   - Body: `{"username": "myuser", "password": "mypassword"}`
//! - `POST /login`: Log in and receive a JWT.
//!   - Body: `{"username": "myuser", "password": "mypassword"}`
//! - `GET /me`: Access a protected route.
//!   - Header: `Authorization: Bearer <your-jwt-token>`

// Import all the good stuff from the RSB prelude
use rsb::prelude::*;
use std::collections::HashMap;

fn main() {
    // Bootstrap RSB to load environment variables like JWT_SECRET
    bootstrap!();
    info!("Starting SaaS example application...");

    // --- Database Setup ---
    // Ensure the users table exists.
    if let Ok(conn) = rsb::db::db_query_rows("SELECT name FROM sqlite_master WHERE type='table' AND name='users';", &[]) {
        if conn.is_empty() {
            info!("'users' table not found, creating it.");
            rsb::db::db_execute("CREATE TABLE users (username TEXT PRIMARY KEY, password_hash TEXT);", &[]).unwrap();
        }
    }

    // --- Web Server Routing ---

    // 1. User Registration Endpoint
    route!("/register", |req: WebRequest| {
        if req.method != "POST" { return "Method Not Allowed".to_string(); }
        let json: Result<HashMap<String, String>, _> = serde_json::from_str(&req.body);
        match json {
            Ok(mut creds) => {
                let username = creds.remove("username").unwrap_or_default();
                let password = creds.remove("password").unwrap_or_default();

                if username.is_empty() || password.is_empty() {
                    return "{\"error\": \"Username and password are required\"}".to_string();
                }

                let hash = password_hash!(&password).unwrap();
                match rsb::db::db_execute("INSERT INTO users (username, password_hash) VALUES (?1, ?2);", &[&username, &hash]) {
                    Ok(_) => "{\"status\": \"ok\"}".to_string(),
                    Err(e) => format!("{{\"error\": \"User already exists: {}\"}}", e),
                }
            },
            Err(_) => "{\"error\": \"Invalid JSON\"}".to_string()
        }
    });

    // 2. User Login Endpoint
    route!("/login", |req: WebRequest| {
        if req.method != "POST" { return "Method Not Allowed".to_string(); }
        let json: Result<HashMap<String, String>, _> = serde_json::from_str(&req.body);
        match json {
            Ok(mut creds) => {
                let username = creds.remove("username").unwrap_or_default();
                let password = creds.remove("password").unwrap_or_default();

                let query = "SELECT password_hash FROM users WHERE username = ?1;";
                match rsb::db::db_query_rows(query, &[&username]) {
                    Ok(rows) if !rows.is_empty() => {
                        let hash = &rows[0]["password_hash"];
                        if password_verify!(&password, hash) {
                            let mut data = HashMap::new();
                            data.insert("user".to_string(), username.clone());
                            let token = jwt_sign!(&username, 24, data).unwrap();
                            format!("{{\"token\": \"{}\"}}", token)
                        } else {
                            "{\"error\": \"Invalid credentials\"}".to_string()
                        }
                    },
                    _ => "{\"error\": \"Invalid credentials\"}".to_string(),
                }
            },
            Err(_) => "{\"error\": \"Invalid JSON\"}".to_string()
        }
    });

    // 3. Protected Endpoint
    route!("/me", |req: WebRequest| {
        let auth_header = req.headers.get("Authorization").unwrap_or(&"".to_string()).to_string();
        if !auth_header.starts_with("Bearer ") {
            return "{\"error\": \"Unauthorized\"}".to_string();
        }
        let token = &auth_header["Bearer ".len()..];
        match jwt_verify!(token) {
            Ok(claims) => format!("{{\"success\": true, \"user_data\": {:?}}}", claims),
            Err(_) => "{\"error\": \"Invalid token\"}".to_string(),
        }
    });

    // --- Start The Server ---
    info!("Server starting on http://0.0.0.0:8008");
    http_server!(8008);
}
