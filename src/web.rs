// src/web.rs

use tiny_http::{Server, Response};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// A simple request struct to pass to handlers
pub struct Request {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

// A global router for simplicity
lazy_static::lazy_static! {
    static ref ROUTES: Arc<Mutex<HashMap<String, Box<dyn Fn(Request) -> String + Send + Sync>>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

/// Registers a handler for a specific route.
pub fn web_route(path: &str, handler: impl Fn(Request) -> String + Send + Sync + 'static) {
    ROUTES.lock().unwrap().insert(path.to_string(), Box::new(handler));
}

/// Starts the web server.
pub fn web_start(port: u16) {
    let addr = format!("0.0.0.0:{}", port);
    let server = Server::http(&addr).unwrap();
    println!("RSB web server listening on {}", addr);

    for mut request in server.incoming_requests() {
        let path = request.url().to_string();
        let method = request.method().to_string();
        let headers = request.headers().iter()
            .map(|h| (h.field.to_string(), h.value.to_string()))
            .collect();
        let mut content = String::new();
        request.as_reader().read_to_string(&mut content).unwrap();

        let req = Request {
            method,
            path: path.clone(),
            headers,
            body: content,
        };

        let routes = ROUTES.lock().unwrap();
        let response = if let Some(handler) = routes.get(&path) {
            Response::from_string(handler(req))
        } else {
            Response::from_string("Not Found").with_status_code(404)
        };
        request.respond(response).unwrap();
    }
}
