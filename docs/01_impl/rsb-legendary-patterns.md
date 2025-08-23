# RSB Legendary Bash Script Patterns

## 1. Heredoc with cat! (Already Solved!)

You're right - `cat!` already handles heredocs beautifully:

```rust
// Bash heredoc equivalent
let config_template = r#"
# Application Config
DEBUG=${DEBUG:-false}
PORT=${PORT:-8080}
DATABASE_URL=${DATABASE_URL:-sqlite://app.db}
"#;

// Write expanded template
cat!(config_template)
    .sed("${DEBUG}", &param!("DEBUG", default: "false"))
    .sed("${PORT}", &param!("PORT", default: "8080"))
    .sed("${DATABASE_URL}", &param!("DATABASE_URL", default: "sqlite://app.db"))
    .to_file("$CONFIG_DIR/app.conf");

// Or direct heredoc-style writing
write_file("script.sh", &var!(r#"#!/bin/bash
echo "Starting $APP_NAME"
cd $WORK_DIR
exec $COMMAND "$@"
"#).expand());
```

## 2. Process Substitution Patterns

Process substitution for complex piping scenarios:

```rust
// Process substitution macros
macro_rules! proc_sub {
    (< $command:expr) => {
        {
            // Create temporary named pipe for input substitution
            let temp_path = format!("/tmp/rsb_proc_{}", std::process::id());
            cmd!($command).to_file(&temp_path);
            temp_path
        }
    };
    
    (> $command:expr) => {
        {
            // Create temporary pipe for output substitution
            let temp_path = format!("/tmp/rsb_out_{}", std::process::id());
            // Command will read from this pipe
            temp_path
        }
    };
}

// Usage examples
// Bash: diff <(ls dir1) <(ls dir2)
let diff_result = shell!(
    "diff {} {}", 
    proc_sub!(< "ls dir1"), 
    proc_sub!(< "ls dir2")
);

// Bash: some_command > >(logger)
let log_pipe = proc_sub!(> "logger -t myapp");
shell!("some_command > {}", log_pipe);
```

## 3. Job Control & Background Processing

Thread-based job management (safer than separate processes):

```rust
use std::sync::{Arc, Mutex};
use std::thread;

// Job control system
lazy_static::lazy_static! {
    static ref JOBS: Arc<Mutex<HashMap<u32, JobHandle>>> = Arc::new(Mutex::new(HashMap::new()));
    static ref JOB_COUNTER: Arc<Mutex<u32>> = Arc::new(Mutex::new(1));
}

pub struct JobHandle {
    pub id: u32,
    pub command: String,
    pub handle: thread::JoinHandle<i32>,
    pub status: JobStatus,
}

pub enum JobStatus {
    Running,
    Completed(i32),
    Failed(String),
}

// Background job macros
macro_rules! job {
    ($command:expr, background) => {
        {
            let job_id = {
                let mut counter = JOB_COUNTER.lock().unwrap();
                *counter += 1;
                *counter
            };
            
            let cmd = $command.to_string();
            let handle = thread::spawn(move || {
                shell!(&cmd, silent) as i32
            });
            
            let job = JobHandle {
                id: job_id,
                command: cmd.clone(),
                handle,
                status: JobStatus::Running,
            };
            
            JOBS.lock().unwrap().insert(job_id, job);
            info!("[{}] Started background job: {}", job_id, cmd);
            job_id
        }
    };
    
    (wait: $job_id:expr) => {
        {
            let mut jobs = JOBS.lock().unwrap();
            if let Some(job) = jobs.remove(&$job_id) {
                match job.handle.join() {
                    Ok(exit_code) => {
                        info!("[{}] Job completed with exit code {}", job.id, exit_code);
                        exit_code
                    },
                    Err(_) => {
                        error!("[{}] Job failed", job.id);
                        1
                    }
                }
            } else {
                error!("Job {} not found", $job_id);
                1
            }
        }
    };
    
    (list) => {
        {
            let jobs = JOBS.lock().unwrap();
            for (id, job) in jobs.iter() {
                echo!("[{}] {} - Running", id, job.command);
            }
        }
    };
}

// Usage examples
let job1 = job!("long_running_task.sh", background);
let job2 = job!("another_task.sh", background);

// Later...
let result1 = job!(wait: job1);
let result2 = job!(wait: job2);

job!(list);  // Show running jobs
```

## 4. Trap Handling for Cleanup

Signal handling and cleanup on exit:

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// Global cleanup registry
lazy_static::lazy_static! {
    static ref CLEANUP_HANDLERS: Arc<Mutex<Vec<Box<dyn Fn() + Send + Sync>>>> = 
        Arc::new(Mutex::new(Vec::new()));
    static ref TRAP_INSTALLED: AtomicBool = AtomicBool::new(false);
}

// Trap registration macro
macro_rules! trap {
    ($handler:expr, on: $signal:expr) => {
        {
            // Install signal handler if not already done
            if !TRAP_INSTALLED.load(Ordering::Relaxed) {
                install_signal_handlers();
                TRAP_INSTALLED.store(true, Ordering::Relaxed);
            }
            
            // Register cleanup handler
            let cleanup_fn = Box::new($handler);
            CLEANUP_HANDLERS.lock().unwrap().push(cleanup_fn);
        }
    };
    
    (cleanup) => {
        {
            let handlers = CLEANUP_HANDLERS.lock().unwrap();
            for handler in handlers.iter() {
                handler();
            }
        }
    };
}

fn install_signal_handlers() {
    extern "C" fn signal_handler(_: i32) {
        trap!(cleanup);
        std::process::exit(130); // SIGINT exit code
    }
    
    unsafe {
        libc::signal(libc::SIGINT, signal_handler as usize);
        libc::signal(libc::SIGTERM, signal_handler as usize);
    }
}

// Usage examples
trap!(|| {
    info!("Cleaning up temporary files...");
    rm_rf("/tmp/myapp_*");
    info!("Cleanup complete");
}, on: "EXIT");

trap!(|| {
    warn!("Received interrupt, stopping gracefully...");
    set_var("INTERRUPTED", "true");
}, on: "INT");
```

## 5. File Piping & Redirection

Enhanced file redirection support:

```rust
// File redirection macros
macro_rules! redirect {
    ($command:expr => $file:expr) => {
        cmd!($command).to_file($file)
    };
    
    ($command:expr >> $file:expr) => {
        cmd!($command).append_to_file($file)
    };
    
    ($command:expr, stdin: $file:expr) => {
        {
            let input = read_file($file);
            let cmd_with_input = format!("echo '{}' | {}", input, $command);
            shell!(&cmd_with_input)
        }
    };
    
    ($command:expr, stderr: $file:expr) => {
        {
            let expanded_cmd = var!($command).expand();
            let expanded_file = var!($file).expand();
            shell!("{} 2> {}", expanded_cmd, expanded_file)
        }
    };
}

// Usage examples
redirect!("ps aux" => "/tmp/processes.txt");
redirect!("find /var/log -name '*.log'" >> "/tmp/all_logs.txt");
redirect!("sort", stdin: "unsorted_data.txt");
redirect!("noisy_command", stderr: "/tmp/errors.log");
```

## 6. QUIET Implementation & Printf Support

BashFX-style QUIET with structured output:

```rust
// Enhanced printf functionality
macro_rules! printf {
    ($format:expr) => {
        print!("{}", var!($format).expand())
    };
    
    ($format:expr, $($args:expr),*) => {
        print!("{}", format!($format, $($args),*))
    };
    
    // printf -v equivalent (store in variable)
    (-v $var:expr, $format:expr) => {
        set_var($var, &var!($format).expand())
    };
    
    (-v $var:expr, $format:expr, $($args:expr),*) => {
        set_var($var, &format!($format, $($args),*))
    };
}

// QUIET-aware output functions
pub fn quiet_echo(level: &str, message: &str) {
    if should_print_level(level) {
        println!("{}", expand_colors(&expand_vars(message)));
    }
}

pub fn quiet_printf(level: &str, format: &str, args: &[&str]) {
    if should_print_level(level) {
        let formatted = args.iter().enumerate().fold(format.to_string(), |acc, (i, arg)| {
            acc.replace(&format!("%{}", i + 1), arg)
        });
        print!("{}", expand_colors(&expand_vars(&formatted)));
    }
}

// Usage examples
printf!("Processing {blue}%s{reset} (%d/%d)\n", "file.txt", "1", "100");
printf!(-v "TIMESTAMP", "$(date '+%Y-%m-%d %H:%M:%S')");
printf!(-v "STATUS_LINE", "Status: {green}%s{reset} | Count: {yellow}%d{reset}", "OK", "42");
```

## 7. Colored Stderr with Glyphs

Visual terminal UX with prefix glyphs:

```rust
// Glyph constants
const GLYPHS: &[(&str, &str)] = &[
    ("info", "ℹ"),     // ℹ️ 
    ("okay", "✓"),     // ✅
    ("warn", "⚠"),     // ⚠️
    ("error", "✗"),    // ❌
    ("fatal", "💀"),   // 💀
    ("debug", "🔍"),   // 🔍
    ("trace", "👁"),   // 👁️
];

// Enhanced stderr with glyphs
pub fn glyph_stderr(level: &str, message: &str) {
    if !should_print_level(level) { return; }
    
    let glyph = GLYPHS.iter()
        .find(|(name, _)| *name == level)
        .map(|(_, g)| g)
        .unwrap_or("•");
        
    let color = match level {
        "info" => "blue",
        "okay" => "green", 
        "warn" => "yellow",
        "error" => "red",
        "fatal" => "red",
        "debug" => "grey",
        "trace" => "grey",
        _ => "reset"
    };
    
    let formatted = format!("{{{}}}{} {}{{{reset}}}", color, glyph, message);
    eprintln!("{}", expand_colors(&expand_vars(&formatted)));
}

// Override existing macros to use glyphs
#[macro_export]
macro_rules! info {
    ($msg:expr) => { glyph_stderr("info", $msg) };
    ($msg:expr, $($args:expr),*) => {
        let formatted = format!($msg, $($args),*);
        glyph_stderr("info", &formatted)
    };
}

// Usage
info!("Starting {blue}$APP_NAME{reset} v{yellow}$VERSION{reset}");
okay!("Configuration loaded from {cyan}$CONFIG_FILE{reset}");
warn!("Using default settings for {yellow}$MISSING_KEY{reset}");
error!("Failed to connect to {red}$DATABASE_URL{reset}");
```

## 8. Developer Functions & Direct Function Calls

Dollar-sign command for testing functions:

```rust
// Developer function registry and dispatcher
lazy_static::lazy_static! {
    static ref DEV_FUNCTIONS: HashMap<&'static str, fn(&Args)> = {
        let mut map = HashMap::new();
        map.insert("test_config", dev_test_config);
        map.insert("dump_vars", dev_dump_vars);
        map.insert("reset_state", dev_reset_state);
        // Add more dev functions as needed
        map
    };
}

// Developer dispatch - callable as: ./script $ function_name arg1 arg2
fn do_dev_call(args: Args) -> i32 {
    require_var!("DEV_MODE"); // Only in dev mode
    
    let func_name = args.get_or(1, "");
    if func_name.is_empty() {
        echo!("Available dev functions:");
        for name in DEV_FUNCTIONS.keys() {
            echo!("  ./script $ {grey}${}{reset}", name);
        }
        return 1;
    }
    
    if let Some(func) = DEV_FUNCTIONS.get(func_name) {
        let func_args = Args::new(&args.all()[2..]);
        info!("🔧 Calling dev function: {cyan}${}{reset}", func_name);
        func(&func_args);
        0
    } else {
        error!("Unknown dev function: {red}${}{reset}", func_name);
        1
    }
}

// Dev function examples
fn dev_test_config(args: &Args) {
    info!("Testing configuration...");
    for key in ["PROJECT", "VERSION", "BUILD_DIR"] {
        if has_var(key) {
            okay!("{}: {green}${}{reset}", key, get_var(key));
        } else {
            warn!("{}: {yellow}not set{reset}", key);
        }
    }
}

fn dev_dump_vars(args: &Args) {
    info!("Current variables:");
    // This would need Context to expose variable iteration
    echo!("PROJECT = $PROJECT");
    echo!("VERSION = $VERSION");
    echo!("DEBUG_MODE = $DEBUG_MODE");
}

// Usage in main dispatch
rsb_dispatch!(args, {
    "build" => do_build,
    "deploy" => do_deploy,
    "$" => do_dev_call  // Special dev dispatcher
});
```

## 9. Ordered File Loading Pattern

Loading numbered files in sequence:

```rust
// Ordered file loading
pub fn load_ordered_files(pattern: &str, extension: &str) -> Vec<String> {
    let expanded_pattern = var!(pattern).expand();
    let dir = std::path::Path::new(&expanded_pattern).parent()
        .unwrap_or(std::path::Path::new("."));
        
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .unwrap_or_else(|_| panic!("Cannot read directory: {}", dir.display()))
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let name = entry.file_name().to_string_lossy().to_string();
            
            // Match pattern: 00_something.ext, 01_another.ext
            if name.ends_with(extension) && 
               name.len() >= 3 && 
               name.chars().take(2).all(|c| c.is_ascii_digit()) &&
               name.chars().nth(2) == Some('_') {
                Some((name.clone(), entry.path()))
            } else {
                None
            }
        })
        .collect();
    
    // Sort by numeric prefix
    files.sort_by(|a, b| a.0.cmp(&b.0));
    
    files.into_iter().map(|(_, path)| {
        path.to_string_lossy().to_string()
    }).collect()
}

// Macro for loading ordered files
macro_rules! load_ordered {
    ($pattern:expr, $extension:expr) => {
        {
            let files = load_ordered_files($pattern, $extension);
            for file in files {
                info!("Loading: {grey}${}{reset}", file);
                match $extension {
                    ".sh" => {
                        // Execute bash files (future feature?)
                        shell!("source {}", file);
                    },
                    ".conf" => {
                        load_config_file(&file);
                    },
                    ".rc" => {
                        load_config_file(&file);
                    },
                    _ => {
                        // Just read content, store in numbered variables
                        let content = read_file(&file);
                        let base_name = std::path::Path::new(&file)
                            .file_stem()
                            .unwrap()
                            .to_string_lossy();
                        set_var(&format!("LOADED_{}", base_name.to_uppercase()), &content);
                    }
                }
            }
        }
    };
}

// Usage examples
load_ordered!("./config/??_*.conf", ".conf");  // 00_base.conf, 01_local.conf, etc.
load_ordered!("./init/??_*.rc", ".rc");        // 00_system.rc, 01_user.rc, etc.

// Manual iteration
let init_files = load_ordered_files("./scripts/??_init_*.sh", ".sh");
for (i, file) in init_files.iter().enumerate() {
    info!("[{}/{}] Executing: {blue}${}{reset}", i+1, init_files.len(), file);
    shell!("bash {}", file);
}
```

This completes the legendary bash script patterns! The combination of all these features should handle virtually any complex bash script conversion to RSB while maintaining the string-first philosophy and bash-like ergonomics.
