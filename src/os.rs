use std::env;
use std::process::{Command, Stdio};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;

// --- OS Information ---
pub fn os_type() -> String { env::consts::OS.to_string() }
pub fn os_family() -> String { env::consts::FAMILY.to_string() }
pub fn os_arch() -> String { env::consts::ARCH.to_string() }
pub fn os_cpus() -> usize { num_cpus::get() }
pub fn os_hostname() -> String { whoami::fallible::hostname().unwrap_or_default() }
pub fn os_homedir() -> String { dirs::home_dir().unwrap_or_default().to_string_lossy().to_string() }
pub fn os_tmpdir() -> String { env::temp_dir().to_string_lossy().to_string() }

// --- Command Execution ---
pub fn is_command(cmd: &str) -> bool {
    which::which(cmd).is_ok()
}

pub fn shell_exec(command: &str, silent: bool) -> Result<String, std::io::Error> {
    let shell = env::var("SHELL").unwrap_or_else(|_| "sh".to_string());
    let mut cmd = Command::new(shell);
    cmd.arg("-c").arg(command);
    if silent {
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
    }
    let output = cmd.output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }
}

// --- Job Control Structures (simplified stubs) ---
pub enum JobStatus { Running, Completed(std::process::ExitStatus) }
pub struct JobHandle {
    pub id: usize,
    pub command: String,
    pub handle: std::thread::JoinHandle<std::process::Output>,
    pub status: Arc<Mutex<JobStatus>>,
}
lazy_static! {
    pub static ref JOBS: Arc<Mutex<HashMap<usize, Arc<Mutex<JobHandle>>>>> = Arc::new(Mutex::new(HashMap::new()));
    pub static ref JOB_COUNTER: Mutex<usize> = Mutex::new(0);
}
pub fn run_cmd_with_status(command: &str) -> std::process::Output {
    let shell = env::var("SHELL").unwrap_or_else(|_| "sh".to_string());
    Command::new(shell).arg("-c").arg(command).output().expect("failed to execute process")
}


// --- Signal/Event Handling Stubs ---
pub struct EventData {
    pub event_type: String,
    pub data: HashMap<String, String>,
}
pub type EventHandler = Box<dyn Fn(&EventData) + Send + Sync>;
lazy_static! {
    pub static ref EVENT_HANDLERS: Arc<Mutex<HashMap<String, Vec<EventHandler>>>> = Arc::new(Mutex::new(HashMap::new()));
}
pub fn install_signal_handlers() {
    // In a real implementation, this would use crates like `ctrlc` or `signal-hook`.
    // For now, it's a placeholder.
}
