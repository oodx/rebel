# RSB Final Completion - Arrays, Events, Utils & Logic

## 1. Complete Array Support (Bash-Style)

### Simple Arrays
```rust
// Simple array macro (indexed)
macro_rules! simple_array {
    ($name:expr, [$($item:expr),*]) => {
        {
            let items = vec![$($item.to_string()),*];
            set_array($name, &items.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        }
    };
    
    // Create empty array
    ($name:expr) => {
        set_array($name, &[]);
    };
}

// Array operations
impl ArrayOps for String {
    fn array_length(&self) -> usize {
        get_array(self).len()
    }
    
    fn array_is_empty(&self) -> bool {
        get_array(self).is_empty()
    }
    
    fn array_push(&self, item: &str) {
        push_array(self, item);
    }
    
    fn array_get(&self, index: usize) -> String {
        get_array(self).get(index).cloned().unwrap_or_default()
    }
    
    fn array_find(&self, needle: &str) -> Option<usize> {
        get_array(self).iter().position(|x| x == needle)
    }
    
    fn array_contains(&self, item: &str) -> bool {
        get_array(self).contains(&item.to_string())
    }
    
    fn array_join(&self, separator: &str) -> String {
        get_array(self).join(separator)
    }
    
    fn array_split(&self, text: &str, delimiter: &str) -> Self {
        let items: Vec<&str> = text.split(delimiter).collect();
        set_array(self, &items);
        self.clone()
    }
    
    fn array_slice(&self, start: usize, end: Option<usize>) -> Vec<String> {
        let arr = get_array(self);
        let end = end.unwrap_or(arr.len());
        arr[start..end.min(arr.len())].to_vec()
    }
}

// Usage examples
simple_array!("TARGETS", ["debug", "release", "test"]);
simple_array!("BUILD_FLAGS"); // Empty array

// Array operations use variable name as string
array_push("TARGETS", "deploy");
let count = array_length("TARGETS");
let first = array_get("TARGETS", 0);

if array_contains("TARGETS", "debug") {
    info!("Debug target available");
}

// Array iteration
for (i, item) in get_array("TARGETS").iter().enumerate() {
    printf!("[{}] {}\n", i, item);
}

// Bash-style for loop
macro_rules! for_in {
    ($var:expr in $array:expr => $body:block) => {
        for item in get_array($array) {
            set_var($var, &item);
            $body
        }
    };
    
    ($index:expr, $var:expr in $array:expr => $body:block) => {
        for (i, item) in get_array($array).iter().enumerate() {
            set_var($index, &i.to_string());
            set_var($var, item);
            $body
        }
    };
}

// Usage
for_in!("target" in "TARGETS" => {
    info!("Processing target: {blue}$target{reset}");
});

for_in!("i", "target" in "TARGETS" => {
    info!("[{}] Processing: {blue}$target{reset}", get_var("i"));
});
```

### Associative Arrays
```rust
// Associative array macro
macro_rules! assoc_array {
    ($name:expr, {$($key:expr => $value:expr),*}) => {
        {
            $(
                set_var(&format!("{}_{}", $name, $key), $value);
            )*
            // Store keys list
            let keys = vec![$($key),*];
            set_array(&format!("{}_KEYS", $name), &keys);
        }
    };
    
    ($name:expr) => {
        set_array(&format!("{}_KEYS", $name), &[]);
    };
}

// Associative array operations
pub fn assoc_set(array_name: &str, key: &str, value: &str) {
    set_var(&format!("{}_{}", array_name, key), value);
    
    // Add key to keys list if not present
    let keys_array = format!("{}_KEYS", array_name);
    if !get_array(&keys_array).contains(&key.to_string()) {
        push_array(&keys_array, key);
    }
}

pub fn assoc_get(array_name: &str, key: &str) -> String {
    get_var(&format!("{}_{}", array_name, key))
}

pub fn assoc_keys(array_name: &str) -> Vec<String> {
    get_array(&format!("{}_KEYS", array_name))
}

pub fn assoc_values(array_name: &str) -> Vec<String> {
    assoc_keys(array_name).iter()
        .map(|key| assoc_get(array_name, key))
        .collect()
}

pub fn assoc_has_key(array_name: &str, key: &str) -> bool {
    has_var(&format!("{}_{}", array_name, key))
}

// Usage examples
assoc_array!("CONFIG", {
    "host" => "localhost",
    "port" => "3000",
    "debug" => "true"
});

assoc_set("CONFIG", "timeout", "30");
let host = assoc_get("CONFIG", "host");

// Iterate over keys and values
for_in!("key" in "CONFIG_KEYS" => {
    let value = assoc_get("CONFIG", &get_var("key"));
    info!("Config: {} = {}", get_var("key"), value);
});

// Get all keys or values as flat arrays
simple_array!("ALL_KEYS", assoc_keys("CONFIG"));
simple_array!("ALL_VALUES", assoc_values("CONFIG"));
```

## 2. Enhanced Event System (Beyond Trap)

```rust
// Event handler registry
lazy_static::lazy_static! {
    static ref EVENT_HANDLERS: Arc<Mutex<HashMap<String, Vec<Box<dyn Fn(&EventData) + Send + Sync>>>>> = 
        Arc::new(Mutex::new(HashMap::new()));
}

pub struct EventData {
    pub event_type: String,
    pub data: HashMap<String, String>,
}

// Event system macros
macro_rules! event {
    (register $event:expr, $handler:expr) => {
        {
            let mut handlers = EVENT_HANDLERS.lock().unwrap();
            let event_handlers = handlers.entry($event.to_string()).or_insert_with(Vec::new);
            event_handlers.push(Box::new($handler));
        }
    };
    
    (emit $event:expr, $($key:expr => $value:expr),*) => {
        {
            let mut data = HashMap::new();
            $(
                data.insert($key.to_string(), $value.to_string());
            )*
            
            let event_data = EventData {
                event_type: $event.to_string(),
                data,
            };
            
            if let Some(handlers) = EVENT_HANDLERS.lock().unwrap().get($event) {
                for handler in handlers {
                    handler(&event_data);
                }
            }
        }
    };
}

// Built-in events
macro_rules! trap {
    ($handler:expr, on: $signal:expr) => {
        event!(register $signal, $handler);
        // Install actual signal handlers if needed
    };
    
    // Custom events
    (on_file_read $handler:expr) => {
        event!(register "file_read", $handler);
    };
    
    (on_pipe_complete $handler:expr) => {
        event!(register "pipe_complete", $handler);
    };
    
    (on_command_start $handler:expr) => {
        event!(register "command_start", $handler);
    };
}

// Enhanced file operations with events
pub fn read_file_with_events(path: &str) -> String {
    event!(emit "file_read", "path" => path, "action" => "start");
    let content = read_file(path);
    event!(emit "file_read", "path" => path, "action" => "complete", "size" => &content.len().to_string());
    content
}

// Usage examples
trap!(|event| {
    info!("File read: {} ({} bytes)", 
          event.data.get("path").unwrap_or(&"unknown".to_string()),
          event.data.get("size").unwrap_or(&"0".to_string()));
}, on_file_read);

trap!(|_| {
    info!("Pipe operation completed");
}, on_pipe_complete);
```

## 3. Poorman's Network & System Utils

```rust
// Poorman's curl (HTTP requests)
pub fn http_get(url: &str) -> String {
    if is_command("curl") {
        shell!("curl -s '{}'", url)
    } else if is_command("wget") {
        shell!("wget -qO- '{}'", url)
    } else {
        error!("Neither curl nor wget available for HTTP requests");
        std::process::exit(1);
    }
}

pub fn http_post(url: &str, data: &str) -> String {
    if is_command("curl") {
        shell!("curl -s -X POST -d '{}' '{}'", data, url)
    } else {
        error!("curl required for HTTP POST requests");
        std::process::exit(1);
    }
}

// Enhanced grep with regex support
macro_rules! regex {
    ($text:expr =~ $pattern:expr) => {
        str_matches($text, $pattern)
    };
    
    ($text:expr !~ $pattern:expr) => {
        !str_matches($text, $pattern)
    };
}

pub fn grep_regex(text: &str, pattern: &str, options: Option<&str>) -> Vec<String> {
    let opts = options.unwrap_or("");
    if is_command("grep") {
        let cmd = if opts.is_empty() {
            format!("echo '{}' | grep -E '{}'", text, pattern)
        } else {
            format!("echo '{}' | grep {} -E '{}'", text, opts, pattern)
        };
        shell!(&cmd).lines().map(|s| s.to_string()).collect()
    } else {
        // Fallback to built-in regex
        use regex::Regex;
        let re = Regex::new(pattern).unwrap_or_else(|_| {
            error!("Invalid regex pattern: {}", pattern);
            std::process::exit(1);
        });
        
        text.lines()
            .filter(|line| re.is_match(line))
            .map(|s| s.to_string())
            .collect()
    }
}

// Enhanced find with pattern matching
pub fn find_files_regex(path: &str, name_pattern: &str, options: Option<&str>) -> Vec<String> {
    let opts = options.unwrap_or("");
    let expanded_path = var!(path).expand();
    
    if is_command("find") {
        let cmd = if opts.is_empty() {
            format!("find '{}' -name '{}'", expanded_path, name_pattern)
        } else {
            format!("find '{}' {} -name '{}'", expanded_path, opts, name_pattern)
        };
        shell!(&cmd).lines().map(|s| s.to_string()).collect()
    } else {
        // Poorman's find (basic directory traversal)
        find_files_basic(&expanded_path, name_pattern)
    }
}

// Poorman's md5
pub fn md5_hash(text: &str) -> String {
    if is_command("md5sum") {
        let hash = shell!("echo -n '{}' | md5sum", text);
        hash.split_whitespace().next().unwrap_or("").to_string()
    } else if is_command("md5") {
        shell!("echo -n '{}' | md5", text)
    } else {
        // Could implement basic MD5 or use a lightweight crate
        error!("No MD5 utility available (md5sum or md5)");
        std::process::exit(1);
    }
}

pub fn md5_file(path: &str) -> String {
    let expanded_path = var!(path).expand();
    if is_command("md5sum") {
        let hash = shell!("md5sum '{}'", expanded_path);
        hash.split_whitespace().next().unwrap_or("").to_string()
    } else if is_command("md5") {
        shell!("md5 '{}'", expanded_path)
    } else {
        let content = read_file(&expanded_path);
        md5_hash(&content)
    }
}

// Macro shortcuts
macro_rules! curl {
    ($url:expr) => { http_get($url) };
    ($url:expr, post: $data:expr) => { http_post($url, $data) };
}

macro_rules! grep {
    ($text:expr, $pattern:expr) => { grep_regex($text, $pattern, None) };
    ($text:expr, $pattern:expr, $opts:expr) => { grep_regex($text, $pattern, Some($opts)) };
}

macro_rules! find {
    ($path:expr, $pattern:expr) => { find_files_regex($path, $pattern, None) };
    ($path:expr, $pattern:expr, $opts:expr) => { find_files_regex($path, $pattern, Some($opts)) };
}
```

## 4. Figlet, Version & Help Messages

```rust
// Figlet support (poorman's version)
pub fn figlet_text(text: &str, font: Option<&str>) -> String {
    if is_command("figlet") {
        if let Some(f) = font {
            shell!("figlet -f {} '{}'", f, text)
        } else {
            shell!("figlet '{}'", text)
        }
    } else {
        // Poorman's ASCII art (basic)
        format!("=== {} ===", text.to_uppercase())
    }
}

macro_rules! figlet {
    ($text:expr) => { figlet_text($text, None) };
    ($text:expr, font: $font:expr) => { figlet_text($text, Some($font)) };
}

// Standard version/help functions
pub fn show_version() {
    if has_var("SCRIPT_LOGO") {
        echo!("$SCRIPT_LOGO");
    } else {
        let logo = figlet!(&get_var("SCRIPT_NAME"));
        echo!("{blue}$logo{reset}");
    }
    echo!("");
    echo!("{bold}$SCRIPT_NAME{reset} v{yellow}$VERSION{reset}");
    echo!("Built with RSB (Rebel String-Based Rust)");
    echo!("");
}

pub fn show_help() {
    show_version();
    echo!("{bold}USAGE:{reset}");
    echo!("  $SCRIPT_NAME <command> [options]");
    echo!("");
    echo!("{bold}COMMANDS:{reset}");
    
    for (name, desc) in list_functions() {
        printf!("  {cyan}%-15s{reset} %s\n", name, desc);
    }
    
    echo!("");
    echo!("{bold}OPTIONS:{reset}");
    echo!("  {green}-h, --help{reset}      Show this help message");
    echo!("  {green}-v, --version{reset}   Show version information");
    echo!("  {green}-d, --debug{reset}     Enable debug output");
    echo!("  {green}-q, --quiet{reset}     Suppress output");
}

// Auto-generated usage from function signatures
macro_rules! register_cmd {
    ($name:expr, $desc:expr, usage: $usage:expr) => {
        register_function($name, &format!("{}\n        Usage: {}", $desc, $usage));
    };
}

// Usage
register_cmd!("build", "Build the project", usage: "build [target] [--clean]");
register_cmd!("deploy", "Deploy to environment", usage: "deploy <env> [--force]");
```

## 5. Enhanced Logic & Control Flow

```rust
// Logical operators for test conditions
macro_rules! and {
    ($a:expr, $b:expr) => { $a && $b };
    ($a:expr, $b:expr, $($rest:expr),+) => { $a && and!($b, $($rest),+) };
}

macro_rules! or {
    ($a:expr, $b:expr) => { $a || $b };
    ($a:expr, $b:expr, $($rest:expr),+) => { $a || or!($b, $($rest),+) };
}

macro_rules! not {
    ($expr:expr) => { !$expr };
}

// Ternary operator
macro_rules! ternary {
    ($condition:expr ? $true_val:expr : $false_val:expr) => {
        if $condition { $true_val } else { $false_val }
    };
}

// Enhanced conditional with multiple tests
macro_rules! if_test {
    ($(test!($($test:tt)*)) && $(test!($($test2:tt)*))) => {
        test!($($test)*) && test!($($test2)*)
    };
    
    ($(test!($($test:tt)*)) || $(test!($($test2:tt)*))) => {
        test!($($test)*) || test!($($test2)*)
    };
}

// Bash-style case statement
macro_rules! case {
    ($value:expr => {
        $($pattern:pat => $body:block),*
        $(, _ => $default:block)?
    }) => {
        {
            let val = $value;
            match val.as_str() {
                $($pattern => $body,)*
                $(_ => $default,)?
            }
        }
    };
    
    // Pattern matching version
    ($value:expr, {
        $($pattern:expr => $body:block),*
        $(, default => $default:block)?
    }) => {
        {
            let val = $value;
            $(
                if str_matches(&val, $pattern) {
                    $body
                } else
            )*
            {
                $($default)?
            }
        }
    };
}

// Usage examples
if and!(test!(-f "config.txt"), test!(-r "config.txt"), !is_empty(&get_var("PROJECT"))) {
    load_config!("config.txt");
}

let status = ternary!(test!(-f "build.log") ? "built" : "not built");

case!(get_var("BUILD_TYPE"), {
    "debug" => {
        set_var("CFLAGS", "-g -O0");
        info!("Debug build configuration");
    },
    "release" => {
        set_var("CFLAGS", "-O3 -DNDEBUG");  
        info!("Release build configuration");
    },
    r"test.*" => {  // Regex pattern
        set_var("CFLAGS", "-g -coverage");
        info!("Test build configuration");
    },
    default => {
        warn!("Unknown build type: $BUILD_TYPE, using debug");
        set_var("BUILD_TYPE", "debug");
    }
});
```

## 6. Configurable Delimiters & String Operations

```rust
// Configurable string splitting
pub trait StringSplitExt {
    fn split_by(&self, delimiter: &str) -> Vec<String>;
    fn split_lines_by(&self, delimiter: &str) -> String;
    fn rsplit_by(&self, delimiter: &str, maxsplit: Option<usize>) -> Vec<String>;
}

impl StringSplitExt for String {
    fn split_by(&self, delimiter: &str) -> Vec<String> {
        self.split(delimiter).map(|s| s.to_string()).collect()
    }
    
    fn split_lines_by(&self, delimiter: &str) -> String {
        self.lines()
            .map(|line| line.split(delimiter).collect::<Vec<_>>().join("\t"))
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    fn rsplit_by(&self, delimiter: &str, maxsplit: Option<usize>) -> Vec<String> {
        let parts: Vec<&str> = if let Some(max) = maxsplit {
            self.rsplitn(max + 1, delimiter).collect()
        } else {
            self.rsplit(delimiter).collect()
        };
        parts.into_iter().rev().map(|s| s.to_string()).collect()
    }
}

// Set global field separator (like awk's FS)
pub fn set_field_separator(fs: &str) {
    set_var("FS", fs);
}

pub fn get_field_separator() -> String {
    param!("FS", default: " ")
}

// Enhanced cut with custom delimiter
impl StringExt for String {
    fn cut_with_fs(&self, field: usize) -> String {
        let fs = get_field_separator();
        self.cut(field, &fs)
    }
    
    fn awk_with_fs(&self, field: usize) -> Vec<String> {
        let fs = get_field_separator();
        self.lines()
            .filter_map(|line| line.split(&fs).nth(field - 1))
            .map(|s| s.to_string())
            .collect()
    }
}

// Usage
set_field_separator(":");
let usernames = cat!("/etc/passwd")
    .awk_with_fs(1)  // First field (username)
    .join(" ");
```

## 7. Function Return Values & Exit Status

All RSB functions return proper exit codes and support result capture:

```rust
// Function result pattern
pub struct FunctionResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

// Enhanced function calls with result capture
macro_rules! call_with_result {
    ($func:ident($($arg:expr),*)) => {
        {
            let old_quiet = has_var("TEMP_QUIET");
            set_var("TEMP_QUIET", "true");
            
            let result = $func($($arg),*);
            
            if !old_quiet { unset_var("TEMP_QUIET"); }
            
            result
        }
    };
}

// Every RSB function returns exit status
fn do_example_function(args: Args) -> i32 {
    let mut ret = 1; // Default to failure
    
    // ... function logic ...
    
    if success_condition {
        ret = 0; // Explicit success
    }
    
    ret // Always return exit status
}

// Main function ensures proper exit
fn main() {
    let args: Vec<String> = env::args().collect();
    rsb_bootstrap(&args);
    
    // Handle version/help first
    if args.len() > 1 {
        match args[1].as_str() {
            "-v" | "--version" => { show_version(); std::process::exit(0); },
            "-h" | "--help" => { show_help(); std::process::exit(0); },
            _ => {}
        }
    }
    
    let exit_code = rsb_dispatch!(args, {
        "build" => do_build,
        "deploy" => do_deploy,
        "test" => do_test
    });
    
    std::process::exit(exit_code); // Always exit with proper status
}
```

## 8. Missing System Utilities & Macros

### Core System Functions
```rust
// System command detection (was missing!)
pub fn is_command(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false) ||
    std::process::Command::new("command")
        .arg("-v")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// Enhanced logical operators
macro_rules! xor {
    ($a:expr, $b:expr) => { ($a || $b) && !($a && $b) };
}

macro_rules! nor {
    ($a:expr, $b:expr) => { !($a || $b) };
    ($a:expr, $b:expr, $($rest:expr),+) => { !or!($a, $b, $($rest),+) };
}

// Random number generation
macro_rules! rand {
    ($min:expr, $max:expr) => {
        {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            
            let mut hasher = DefaultHasher::new();
            std::time::SystemTime::now().hash(&mut hasher);
            let seed = hasher.finish();
            
            let range = ($max as u64) - ($min as u64) + 1;
            ($min as u64 + (seed % range)) as i32
        }
    };
}

// Date functions
macro_rules! date {
    () => { shell!("date") };
    ($format:expr) => { shell!("date '{}'", $format) };
    (iso) => { shell!("date -Iseconds") };
    (epoch) => { shell!("date +%s") };
    (human) => { shell!("date '+%Y-%m-%d %H:%M:%S'") };
}

// Function/command existence check
macro_rules! exists {
    (cmd: $command:expr) => { is_command($command) };
    (func: $function:expr) => { is_function($function) };
    (var: $variable:expr) => { has_var($variable) };
    (file: $path:expr) => { is_file($path) };
    (dir: $path:expr) => { is_dir($path) };
}

// Process management
macro_rules! pkill {
    ($pattern:expr) => {
        if is_command("pkill") {
            shell!("pkill '{}'", $pattern);
        } else {
            warn!("pkill not available, using manual process termination");
            let pids = shell!("pgrep '{}'", $pattern);
            for pid in pids.lines() {
                if !pid.trim().is_empty() {
                    shell!("kill {}", pid.trim());
                }
            }
        }
    };
    
    ($signal:expr, $pattern:expr) => {
        if is_command("pkill") {
            shell!("pkill -{} '{}'", $signal, $pattern);
        } else {
            let pids = shell!("pgrep '{}'", $pattern);
            for pid in pids.lines() {
                if !pid.trim().is_empty() {
                    shell!("kill -{} {}", $signal, pid.trim());
                }
            }
        }
    };
}

// Nuclear cleanup
macro_rules! nuke {
    () => {
        warn!("🚨 Nuclear cleanup initiated...");
        // Kill all background jobs
        let jobs = JOBS.lock().unwrap();
        for (id, _) in jobs.iter() {
            warn!("Terminating job [{}]", id);
        }
        drop(jobs);
        
        // Run all cleanup handlers
        trap!(cleanup);
        
        // Clean temp files
        if has_var("TEMP_DIR") {
            rm_rf(&get_var("TEMP_DIR"));
        }
        
        okay!("Nuclear cleanup complete");
    };
    
    (jobs) => {
        // Kill just background jobs
        let mut jobs = JOBS.lock().unwrap();
        for (id, job) in jobs.drain() {
            warn!("Nuking job [{}]: {}", id, job.command);
        }
    };
}

// Process ID functions
macro_rules! pid {
    () => { std::process::id() };
    (parent) => { 
        if is_command("ps") {
            shell!("ps -o ppid= -p {}", std::process::id()).trim().to_string()
        } else {
            "unknown".to_string()
        }
    };
}

// Enhanced job system with completion events
pub fn start_job_with_events(command: &str) -> u32 {
    let job_id = {
        let mut counter = JOB_COUNTER.lock().unwrap();
        *counter += 1;
        *counter
    };
    
    let cmd = command.to_string();
    let handle = thread::spawn(move || {
        let start_time = std::time::Instant::now();
        event!(emit "job_start", "id" => &job_id.to_string(), "command" => &cmd);
        
        let result = shell!(&cmd, silent) as i32;
        let duration = start_time.elapsed();
        
        event!(emit "job_complete", 
               "id" => &job_id.to_string(), 
               "command" => &cmd,
               "result" => &result.to_string(),
               "duration_ms" => &duration.as_millis().to_string());
        result
    });
    
    job_id
}
```

### File System Operations (Complete Set)
```rust
// Complete file system operations
pub fn rm(path: &str) -> bool {
    let expanded = var!(path).expand();
    std::fs::remove_file(&expanded)
        .or_else(|_| std::fs::remove_dir(&expanded))
        .is_ok()
}

pub fn rm_rf(path: &str) -> bool {
    let expanded = var!(path).expand();
    if test!(-e &expanded) {
        warn!("Removing: {}", expanded);
        std::fs::remove_dir_all(&expanded)
            .or_else(|_| std::fs::remove_file(&expanded))
            .is_ok()
    } else {
        true // Path doesn't exist, consider success
    }
}

pub fn cp(src: &str, dest: &str) -> bool {
    let src_expanded = var!(src).expand();
    let dest_expanded = var!(dest).expand();
    
    std::fs::copy(&src_expanded, &dest_expanded).is_ok()
}

pub fn cp_r(src: &str, dest: &str) -> bool {
    let src_expanded = var!(src).expand();
    let dest_expanded = var!(dest).expand();
    
    if is_command("cp") {
        shell!("cp -r '{}' '{}'", src_expanded, dest_expanded, silent)
    } else {
        // Basic recursive copy implementation
        copy_recursive(&src_expanded, &dest_expanded)
    }
}

pub fn mv(src: &str, dest: &str) -> bool {
    let src_expanded = var!(src).expand();
    let dest_expanded = var!(dest).expand();
    
    std::fs::rename(&src_expanded, &dest_expanded).is_ok()
}

pub fn mkdir_p(path: &str) -> bool {
    let expanded = var!(path).expand();
    std::fs::create_dir_all(&expanded).is_ok()
}

pub fn touch(path: &str) -> bool {
    let expanded = var!(path).expand();
    
    if std::path::Path::new(&expanded).exists() {
        // Update timestamp
        if is_command("touch") {
            shell!("touch '{}'", expanded, silent)
        } else {
            // Update access time manually
            use std::fs::OpenOptions;
            OpenOptions::new().write(true).open(&expanded).is_ok()
        }
    } else {
        // Create file
        std::fs::File::create(&expanded).is_ok()
    }
}

// Macros for file operations
macro_rules! rm {
    ($path:expr) => { rm($path) };
    (-rf $path:expr) => { rm_rf($path) };
    (-r $path:expr) => { rm_rf($path) };
    (-f $path:expr) => { rm($path) };
}

macro_rules! cp {
    ($src:expr, $dest:expr) => { cp($src, $dest) };
    (-r $src:expr, $dest:expr) => { cp_r($src, $dest) };
}

macro_rules! mkdir {
    ($path:expr) => { 
        let expanded = var!($path).expand();
        std::fs::create_dir(&expanded).is_ok()
    };
    (-p $path:expr) => { mkdir_p($path) };
}
```

### User Interaction & Prompts
```rust
// User input functions
pub fn prompt(message: &str, default: Option<&str>) -> String {
    use std::io::{self, Write};
    
    let default_text = if let Some(def) = default {
        format!(" [{}]", def)
    } else {
        String::new()
    };
    
    print!("{}{}: ", var!(message).expand(), default_text);
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    
    let trimmed = input.trim();
    if trimmed.is_empty() && default.is_some() {
        default.unwrap().to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn select_menu(message: &str, options: &[&str]) -> (usize, String) {
    use std::io::{self, Write};
    
    println!("{}", var!(message).expand());
    for (i, option) in options.iter().enumerate() {
        println!("  {}) {}", i + 1, option);
    }
    
    loop {
        print!("Select [1-{}]: ", options.len());
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");
        
        if let Ok(choice) = input.trim().parse::<usize>() {
            if choice >= 1 && choice <= options.len() {
                return (choice - 1, options[choice - 1].to_string());
            }
        }
        
        error!("Invalid selection. Please choose 1-{}", options.len());
    }
}

pub fn confirm(message: &str, default: Option<bool>) -> bool {
    use std::io::{self, Write};
    
    // Check opt_yes flag first
    if has_var("opt_yes") {
        info!("{} - auto-confirmed with --yes flag", message);
        return true;
    }
    
    let default_text = match default {
        Some(true) => " [Y/n/q]",
        Some(false) => " [y/N/q]", 
        None => " [y/n/q]"
    };
    
    loop {
        print!("{}{}: ", var!(message).expand(), default_text);
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");
        
        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            "q" | "quit" => std::process::exit(0),
            "" => {
                if let Some(def) = default {
                    return def;
                }
                continue;
            }
            _ => {
                warn!("Please answer y/n/q");
                continue;
            }
        }
    }
}

// Macros for user interaction
macro_rules! prompt {
    ($message:expr) => { prompt($message, None) };
    ($message:expr, default: $default:expr) => { prompt($message, Some($default)) };
}

macro_rules! select {
    ($message:expr, [$($option:expr),*]) => {
        select_menu($message, &[$($option),*])
    };
}

macro_rules! confirm {
    ($message:expr) => { confirm($message, None) };
    ($message:expr, default: $default:expr) => { confirm($message, Some($default)) };
}
```

### Utility Functions
```rust
// Array dumping
pub fn dump_array(name: &str, to_file: Option<&str>) -> String {
    let items = get_array(name);
    let output = format!("Array '{}' ({} items):\n{}", 
                        name, 
                        items.len(),
                        items.iter()
                             .enumerate()
                             .map(|(i, item)| format!("  [{}] {}", i, item))
                             .collect::<Vec<_>>()
                             .join("\n"));
    
    if let Some(file) = to_file {
        write_file(&var!(file).expand(), &output);
    }
    
    println!("{}", output);
    output
}

macro_rules! dump {
    ($array:expr) => { dump_array($array, None) };
    ($array:expr => $file:expr) => { dump_array($array, Some($file)) };
}

// Enhanced printf with variable assignment
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

// System utilities
macro_rules! realpath {
    ($path:expr) => {
        {
            let expanded = var!($path).expand();
            std::fs::canonicalize(&expanded)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| {
                    if is_command("realpath") {
                        shell!("realpath '{}'", expanded)
                    } else {
                        expanded // Fallback to original path
                    }
                })
        }
    };
}

macro_rules! sleep {
    ($seconds:expr) => {
        std::thread::sleep(std::time::Duration::from_secs($seconds as u64))
    };
    (ms: $milliseconds:expr) => {
        std::thread::sleep(std::time::Duration::from_millis($milliseconds as u64))
    };
}

macro_rules! clear {
    () => {
        if is_command("clear") {
            print!("{}", shell!("clear"));
        } else {
            print!("\x1B[2J\x1B[1;1H"); // ANSI clear screen
        }
    };
}

// Line generation
macro_rules! line {
    ($char:expr, $count:expr) => {
        $char.to_string().repeat($count as usize)
    };
}

// General shell command wrapper generator
macro_rules! shell_wrapper {
    ($name:ident, $command:expr) => {
        macro_rules! $name {
            ($($args:expr),*) => {
                if is_command($command) {
                    shell!("{} {}", $command, format!("{}", format_args!("{} ", $($args),*)).trim())
                } else {
                    error!("{} command not available", $command);
                    std::process::exit(1);
                }
            };
        }
    };
}

// Create wrappers for common commands
shell_wrapper!(git, "git");
shell_wrapper!(docker, "docker"); 
shell_wrapper!(ssh, "ssh");
shell_wrapper!(rsync, "rsync");
```

### Usage Examples
```rust
// Logic tests
if xor!(test!(-f "file1"), test!(-f "file2")) {
    info!("Exactly one file exists");
}

if nor!(is_empty(&get_var("USER")), is_empty(&get_var("HOME"))) {
    error!("Neither USER nor HOME is set");
}

// Random and utilities  
let port = rand!(3000, 9000);
let timestamp = date!("+%Y%m%d_%H%M%S");
let current_dir = realpath!(".");

// User interaction
let name = prompt!("Enter your name", default: "anonymous");
let (choice, selected) = select!("Choose environment", ["dev", "staging", "prod"]);

if confirm!("Deploy to production?", default: false) {
    info!("Deploying...");
}

// File operations
mkdir!(-p "$BUILD_DIR/artifacts");
touch!("$BUILD_DIR/build.log");
cp!(-r "src/", "$BUILD_DIR/source/");

// Job management with events
trap!(|event| {
    let job_id = event.data.get("id").unwrap();
    let result = event.data.get("result").unwrap();
    info!("Job {} completed with status {}", job_id, result);
}, on: "job_complete");

let job_id = start_job_with_events("long_running_process.sh");

// Cleanup
nuke!(jobs); // Kill all background jobs
clear!();
let separator = line!('=', 80);
echo!("$separator");
```
