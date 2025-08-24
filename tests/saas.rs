use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

#[test]
fn test_saas_example_flow() {
    // Build the example
    let build_status = Command::new("cargo")
        .args(["build", "--example", "saas_dashboard"])
        .status()
        .expect("Failed to build saas_dashboard example");
    assert!(build_status.success());

    // Run the server in the background
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let binary_path = std::path::Path::new(&manifest_dir)
        .join("target/debug/examples/saas_dashboard");
    let mut server_process = Command::new(binary_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start saas_dashboard server");

    // Give the server a moment to start
    thread::sleep(Duration::from_secs(2));

    // --- Test /register (This is a placeholder, as the handler is hardcoded) ---
    let reg_res = ureq::post("http://localhost:8000/register").call().unwrap();
    assert_eq!(reg_res.status(), 200);
    assert_eq!(reg_res.into_string().unwrap(), "User registered");

    // --- Test /login ---
    let login_res = ureq::post("http://localhost:8000/login").call().unwrap();
    assert_eq!(login_res.status(), 200);
    let token = login_res.into_string().unwrap();
    assert!(!token.is_empty() && token != "Invalid login");

    // --- Test /api/data (protected) ---
    // Without token
    let data_res_fail = ureq::get("http://localhost:8000/api/data").call().unwrap();
    assert_eq!(data_res_fail.status(), 200); // The server returns 200, but body is "Unauthorized"
    assert_eq!(data_res_fail.into_string().unwrap(), "Unauthorized");

    // With token
    let data_res_ok = ureq::get("http://localhost:8000/api/data")
        .set("Authorization", &format!("Bearer {}", token))
        .call()
        .unwrap();
    assert_eq!(data_res_ok.status(), 200);
    assert_eq!(data_res_ok.into_string().unwrap(), "Here is your secret data!");

    // Clean up the server process
    server_process.kill().unwrap();
}
