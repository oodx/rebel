// src/os.rs

// This module will contain operating system interactions, such as
// running commands, getting system info, job control, and trap handling.

// --- Command Execution ---
use crate::context::expand_vars;
use std::process::Command;

/// The result of a command execution, containing status, stdout, and stderr.
#[derive(Debug, Clone)]
pub struct CmdResult {
    pub status: i32,
    pub output: String,
    pub error: String,
}

/// Executes a shell command and returns a `CmdResult`.
pub fn run_cmd_with_status(cmd: &str) -> CmdResult {
    let expanded_cmd = expand_vars(cmd);
    let output = Command::new("sh")
        .arg("-c")
        .arg(&expanded_cmd)
        .output();

    match output {
        Ok(out) => CmdResult {
            status: out.status.code().unwrap_or(1),
            output: String::from_utf8_lossy(&out.stdout).to_string(),
            error: String::from_utf8_lossy(&out.stderr).to_string(),
        },
        Err(e) => CmdResult {
            status: 1,
            output: String::new(),
            error: e.to_string(),
        },
    }
}

/// Executes a shell command and returns its stdout, panicking on error.
pub fn run_cmd(cmd: &str) -> String {
    let result = run_cmd_with_status(cmd);
    if result.status != 0 {
        // Using the error! macro from the crate itself to report.
        // This is a bit recursive, but it's the RSB way.
        eprintln!("Command failed: {}", cmd);
        eprintln!("Stderr: {}", result.error);
        std::process::exit(result.status);
    }
    result.output
}

/// Executes a shell command and captures its output, similar to `$(...)` in bash.
pub fn shell_exec(cmd: &str, silent: bool) -> Result<String, String> {
    let expanded_cmd = expand_vars(cmd);
    let output = Command::new("sh")
        .arg("-c")
        .arg(&expanded_cmd)
        .output();

    match output {
        Ok(out) => {
            if out.status.success() {
                Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
            } else {
                if !silent {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    return Err(format!("Command failed with status {}: {}", out.status, stderr));
                }
                Err(format!("Command failed with status {}", out.status))
            }
        }
        Err(e) => Err(e.to_string()),
    }
}


// --- Job Control ---

use std::thread;

pub enum JobStatus {
    Running,
    Completed(i32),
}

pub struct JobHandle {
    pub id: u32,
    pub command: String,
    pub handle: thread::JoinHandle<CmdResult>,
    pub status: Arc<Mutex<JobStatus>>,
}

lazy_static! {
    pub(crate) static ref JOBS: Arc<Mutex<HashMap<u32, Arc<Mutex<JobHandle>>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    pub(crate) static ref JOB_COUNTER: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
}


// --- Event System ---
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

// --- Signal Handling ---
static TRAP_INSTALLED: AtomicBool = AtomicBool::new(false);

/// The actual C-style signal handler.
extern "C" fn signal_handler(signal: i32) {
    let event_name = match signal {
        libc::SIGINT => "SIGINT",
        libc::SIGTERM => "SIGTERM",
        _ => "UNKNOWN_SIGNAL",
    };
    // It's not safe to do much in a signal handler.
    // We can't allocate memory (like creating EventData) or lock mutexes.
    // The best we can do is set a flag and have the main loop check it.
    // For now, we will just print a message and exit, which is one of the few safe operations.
    eprintln!("\nrsb-trap: Caught signal {}, exiting.", event_name);
    // A more robust implementation would use a global atomic flag.
    std::process::exit(128 + signal);
}

/// Installs the signal handlers for common termination signals.
/// This is done only once.
pub fn install_signal_handlers() {
    if TRAP_INSTALLED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        unsafe {
            libc::signal(libc::SIGINT, signal_handler as usize);
            libc::signal(libc::SIGTERM, signal_handler as usize);
        }
    }
}

#[derive(Debug, Clone)]
pub struct EventData {
    pub event_type: String,
    pub data: HashMap<String, String>,
}

lazy_static! {
    // A registry for event handlers.
    pub(crate) static ref EVENT_HANDLERS: Arc<Mutex<HashMap<String, Vec<Box<dyn Fn(&EventData) + Send + Sync>>>>> =
        Arc::new(Mutex::new(HashMap::new()));
}


// --- System Information ---

/// Gets the current user's name from the context (`USER` variable).
pub fn get_user() -> String {
    crate::context::get_var("USER")
}

/// Gets the current user's home directory from the context (`HOME` variable).
pub fn get_home() -> String {
    crate::context::get_var("HOME")
}

/// Gets the current working directory from the context (`PWD` variable).
pub fn get_pwd() -> String {
    crate::context::get_var("PWD")
}

/// Gets the system's hostname.
pub fn get_hostname() -> String {
    if let Ok(name) = std::process::Command::new("hostname").output() {
        String::from_utf8_lossy(&name.stdout).trim().to_string()
    } else {
        "localhost".to_string()
    }
}

/// Gets the system's architecture (e.g., `x86_64`, `aarch64`).
pub fn get_arch() -> String {
    std::env::consts::ARCH.to_string()
}

/// Gets the operating system (e.g., `linux`, `macos`, `windows`).
pub fn get_os() -> String {
    std::env::consts::OS.to_string()
}


// --- OS Test Functions ---

/// Checks if a command is available in the system's PATH.
pub fn is_command(cmd: &str) -> bool {
    // Try `which` first, as it's common.
    if std::process::Command::new("which")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        return true;
    }

    // Fallback to `command -v`, which is a POSIX standard.
    if std::process::Command::new("command")
        .arg("-v")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        return true;
    }

    false
}
