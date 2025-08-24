use rsb::prelude::*;
use rsb::web::Request;
use std::collections::HashMap;

fn main() {
    bootstrap!();
    set_var("JWT_SECRET", "my-super-secret-key-that-is-long");

    // Initialize the database
    if let Err(e) = db_exec("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, email TEXT UNIQUE, password_hash TEXT)", &[]) {
        fatal!("Failed to initialize database: {}", e);
        std::process::exit(1);
    }

    // Set up web routes
    web_route("/register", handle_register);
    web_route("/login", handle_login);
    web_route("/api/data", handle_api_data);

    // Start the server
    info!("Starting SaaS dashboard on port 8000...");
    web_start(8000);
}

fn handle_register(req: Request) -> String {
    // In a real app, you'd parse the body properly
    let email = "test@example.com";
    let password = "password123";

    let hash = auth_hash_password(password).unwrap();
    match db_exec("INSERT INTO users (email, password_hash) VALUES (?1, ?2)", &[email, &hash]) {
        Ok(_) => "User registered".to_string(),
        Err(e) => format!("Error: {}", e),
    }
}

fn handle_login(req: Request) -> String {
    let email = "test@example.com";
    let password = "password123";

    if let Ok(users) = db_query("SELECT * FROM users WHERE email = ?1", &[email]) {
        if let Some(user) = users.first() {
            if let Some(hash) = user.get("password_hash") {
                if auth_verify_password(password, hash) {
                    if let Some(user_id) = user.get("id") {
                        if let Ok(token) = auth_jwt_create(user_id, 24, HashMap::new()) {
                            return token;
                        }
                    }
                }
            }
        }
    }
    "Invalid login".to_string()
}

fn handle_api_data(req: Request) -> String {
    if let Some(auth_header) = req.headers.get("Authorization") {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            if auth_jwt_verify(token).is_ok() {
                return "Here is your secret data!".to_string();
            }
        }
    }
    "Unauthorized".to_string()
}
