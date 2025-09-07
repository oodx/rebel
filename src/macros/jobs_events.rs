// --- Job Control Macros ---
// Namespaced re-exports for selective imports
pub use crate::{job, event, trap, hostname, user, home_dir, current_dir, curl, get, pid_of, process_exists, kill_pid, kill_process, with_lock, lock, unlock};
#[macro_export]
macro_rules! job {
    (background: $command:expr) => {{
        let mut counter = $crate::os::JOB_COUNTER.lock().unwrap();
        *counter += 1;
        let job_id = *counter;
        let cmd_string = $command.to_string();
        let (tx, rx) = std::sync::mpsc::channel();
        let cmd_string_for_thread = cmd_string.clone();

        let handle = std::thread::spawn(move || {
            let result = $crate::os::run_cmd_with_status(&cmd_string_for_thread);
            let _ = tx.send(result);
        });

        let job_handle = $crate::os::JobHandle {
            id: job_id,
            command: cmd_string,
            handle: Some(handle),
            rx: rx,
        };
        $crate::os::JOBS.lock().unwrap().insert(job_id, std::sync::Arc::new(std::sync::Mutex::new(job_handle)));
        $crate::info!("[{}] Started background job", job_id);
        job_id
    }};
    (wait: $job_id:expr) => {{
        $crate::info!("[{}] Waiting for job to complete...", $job_id);
        match $crate::os::wait_on_job($job_id, None) {
            Ok(result) => result.status,
            Err(e) => {
                $crate::error!("Failed to wait for job {}: {}", $job_id, e);
                -1
            }
        }
    }};
    (timeout: $timeout:expr, wait: $job_id:expr) => {{
        $crate::info!("[{}] Waiting for job to complete (timeout: {}s)...", $job_id, $timeout);
        let timeout_duration = std::time::Duration::from_secs($timeout);
        match $crate::os::wait_on_job($job_id, Some(timeout_duration)) {
            Ok(result) => result.status,
            Err(e) => {
                $crate::error!("Failed to wait for job {}: {}", $job_id, e);
                -1
            }
        }
    }};
    (list) => {{
        let jobs = $crate::os::JOBS.lock().unwrap();
        if jobs.is_empty() {
            $crate::info!("No running jobs.");
        } else {
            $crate::info!("Running jobs:");
            for (id, job_mutex) in jobs.iter() {
                let job = job_mutex.lock().unwrap();
                $crate::echo!("[{}] {}", id, job.command);
            }
        }
    }};
}

// --- Event Macros ---
#[macro_export]
macro_rules! event {
    (register $event:expr, $handler:expr) => {{
        let mut handlers = $crate::os::EVENT_HANDLERS.lock().unwrap();
        let event_handlers = handlers.entry($event.to_string()).or_insert_with(Vec::new);
        event_handlers.push(Box::new($handler));
    }};
    (emit $event:expr, $($key:expr => $value:expr),*) => {{
        let mut data = ::std::collections::HashMap::new();
        $( data.insert($key.to_string(), $value.to_string()); )*
        let event_data = $crate::os::EventData { event_type: $event.to_string(), data, };
        if let Some(handlers) = $crate::os::EVENT_HANDLERS.lock().unwrap().get($event) {
            for handler in handlers {
                handler(&event_data);
            }
        }
    }};
}

#[macro_export]
macro_rules! trap {
    ($handler:expr, on: $signal:expr) => {{
        let sig_name = $signal.to_uppercase();
        match sig_name.as_str() {
            "SIGINT" | "SIGTERM" | "EXIT" | "COMMAND_ERROR" => {
                $crate::os::install_signal_handlers();
                $crate::event!(register &sig_name, $handler);
            }
            _ => { $crate::event!(register &sig_name, $handler); }
        }
    }};
    (on_file_read $handler:expr) => { $crate::event!(register "file_read", $handler); };
    (on_pipe_complete $handler:expr) => { $crate::event!(register "pipe_complete", $handler); };
    (on_command_start $handler:expr) => { $crate::event!(register "command_start", $handler); };
}

// --- System Info, Network, Process, Locking Macros ---
#[macro_export]
macro_rules! hostname { () => { $crate::os::get_hostname() }; }
#[macro_export]
macro_rules! user { () => { $crate::os::get_username() }; }
#[macro_export]
macro_rules! home_dir { () => { $crate::os::get_home_dir() }; }
#[macro_export]
macro_rules! current_dir { () => { $crate::os::get_current_dir() }; }

#[macro_export]
macro_rules! curl {
    ($url:expr) => {{
        match $crate::os::http_get($url) {
            result if result.status == 0 => result.output,
            result => {
                $crate::error!("curl failed: {}", result.error);
                std::process::exit(result.status);
            }
        }
    }};
    ($url:expr, options: $opts:expr) => {{
        match $crate::os::http_get_with_options($url, $opts) {
            result if result.status == 0 => result.output,
            result => {
                $crate::error!("curl failed: {}", result.error);
                std::process::exit(result.status);
            }
        }
    }};
    (post: $url:expr, data: $data:expr) => {{
        match $crate::os::http_post($url, $data) {
            result if result.status == 0 => result.output,
            result => {
                $crate::error!("curl POST failed: {}", result.error);
                std::process::exit(result.status);
            }
        }
    }};
}

#[macro_export]
macro_rules! get {
    ($url:expr) => { $crate::curl!($url) };
    ($url:expr, options: $opts:expr) => { $crate::curl!($url, options: $opts) };
}

#[macro_export]
macro_rules! pid_of { ($process:expr) => { $crate::os::pid_of($process) }; }
#[macro_export]
macro_rules! process_exists { ($process:expr) => { $crate::os::process_exists($process) }; }

#[macro_export]
macro_rules! kill_pid {
    ($pid:expr) => {{
        match $crate::os::kill_pid($pid, None) {
            result if result.status == 0 => { $crate::okay!("Process {} terminated", $pid); },
            result => { $crate::error!("Failed to kill process {}: {}", $pid, result.error); std::process::exit(result.status); }
        }
    }};
    ($pid:expr, signal: $sig:expr) => {{
        match $crate::os::kill_pid($pid, Some($sig)) {
            result if result.status == 0 => { $crate::okay!("Process {} terminated with {}", $pid, $sig); },
            result => { $crate::error!("Failed to kill process {}: {}", $pid, result.error); std::process::exit(result.status); }
        }
    }};
}

#[macro_export]
macro_rules! kill_process {
    ($process:expr) => {{
        match $crate::os::kill_process($process, None) {
            result if result.status == 0 => { $crate::okay!("Killed all {} processes", $process); },
            result => { $crate::error!("Failed to kill {}: {}", $process, result.error); std::process::exit(result.status); }
        }
    }};
    ($process:expr, signal: $sig:expr) => {{
        match $crate::os::kill_process($process, Some($sig)) {
            result if result.status == 0 => { $crate::okay!("Killed all {} processes with {}", $process, $sig); },
            result => { $crate::error!("Failed to kill {}: {}", $process, result.error); std::process::exit(result.status); }
        }
    }};
}

// --- Locking Macros ---
#[macro_export]
macro_rules! with_lock {
    ($lock_path:expr => $body:block) => {{
        match $crate::os::create_lock($lock_path) {
            Ok(_) => { let result = $body; $crate::os::remove_lock($lock_path); result },
            Err(e) => { $crate::error!("Failed to acquire lock: {}", e); std::process::exit(1); }
        }
    }};
}

#[macro_export]
macro_rules! lock {
    ($lock_path:expr) => {{
        match $crate::os::create_lock($lock_path) {
            Ok(_) => { $crate::okay!("Lock acquired: {}", $lock_path); },
            Err(e) => { $crate::error!("Failed to acquire lock: {}", e); std::process::exit(1); }
        }
    }};
}

#[macro_export]
macro_rules! unlock { ($lock_path:expr) => { $crate::os::remove_lock($lock_path); $crate::okay!("Lock released: {}", $lock_path); }; }
