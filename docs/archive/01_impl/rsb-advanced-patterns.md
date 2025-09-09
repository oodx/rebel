# RSB Advanced Patterns - Macros, Streaming & Command Substitution

## 1. Enhanced Parameter Expansion with Macros

Building on the `param!` macro concept with full bash compatibility:

```rust
// Complete parameter expansion macro
macro_rules! param {
    // Basic patterns
    ($var:expr) => { get_var($var) };
    ($var:expr, default: $default:expr) => { 
        if has_var($var) && !is_empty(&get_var($var)) { 
            get_var($var) 
        } else { 
            $default.to_string() 
        }
    };
    ($var:expr, alt: $alt:expr) => { 
        if has_var($var) && !is_empty(&get_var($var)) { 
            $alt.to_string() 
        } else { 
            String::new() 
        }
    };
    
    // String manipulation patterns
    ($var:expr, sub: $start:expr) => { var_substring(&get_var($var), $start, None) };
    ($var:expr, sub: $start:expr, $len:expr) => { var_substring(&get_var($var), $start, Some($len)) };
    ($var:expr, prefix: $pattern:expr) => { var_trim_prefix(&get_var($var), $pattern, false) };
    ($var:expr, prefix: $pattern:expr, longest) => { var_trim_prefix(&get_var($var), $pattern, true) };
    ($var:expr, suffix: $pattern:expr) => { var_trim_suffix(&get_var($var), $pattern, false) };
    ($var:expr, suffix: $pattern:expr, longest) => { var_trim_suffix(&get_var($var), $pattern, true) };
    ($var:expr, replace: $from:expr => $to:expr) => { var_replace(&get_var($var), $from, $to, false) };
    ($var:expr, replace: $from:expr => $to:expr, all) => { var_replace(&get_var($var), $from, $to, true) };
    
    // Case conversion
    ($var:expr, upper) => { get_var($var).to_uppercase() };
    ($var:expr, lower) => { get_var($var).to_lowercase() };
    ($var:expr, upper: first) => { var_case_upper(&get_var($var), false) };
    ($var:expr, lower: first) => { var_case_lower(&get_var($var), false) };
    
    // Length and tests
    ($var:expr, len) => { get_var($var).len() };
}

// Usage examples that look just like bash
let config = param!("CONFIG_FILE", default: "/etc/app.conf");
let basename = param!("FILEPATH", suffix: ".txt");
let clean_path = param!("PATH", replace: "//" => "/", all);
let first_word = param!("SENTENCE", sub: 0, 10);
let without_prefix = param!("FILENAME", prefix: "backup_");

// Bash: ${VAR:-default} becomes: param!("VAR", default: "default")
// Bash: ${VAR%.*} becomes: param!("VAR", suffix: ".*")
// Bash: ${VAR//old/new} becomes: param!("VAR", replace: "old" => "new", all)
```

## 2. Enhanced Test Conditions with Full Bash Compatibility

```rust
// Complete test macro covering all bash conditions
macro_rules! test {
    // File tests
    (-e $path:expr) => { is_entity($path) };      // exists
    (-f $path:expr) => { is_file($path) };        // regular file
    (-d $path:expr) => { is_dir($path) };         // directory
    (-L $path:expr) => { is_link($path) };        // symbolic link
    (-r $path:expr) => { is_readable($path) };    // readable
    (-w $path:expr) => { is_writable($path) };    // writable
    (-x $path:expr) => { is_executable($path) };  // executable
    (-s $path:expr) => { is_nonempty_file($path) }; // non-empty file
    
    // String tests  
    (-n $str:expr) => { !is_empty($str) };        // non-empty string
    (-z $str:expr) => { is_empty($str) };         // empty string
    
    // String comparisons
    ($a:expr == $b:expr) => { str_equals($a, $b) };
    ($a:expr != $b:expr) => { !str_equals($a, $b) };
    ($a:expr =~ $pattern:expr) => { str_matches($a, $pattern) };
    ($a:expr !~ $pattern:expr) => { !str_matches($a, $pattern) };
    ($a:expr < $b:expr) => { $a < $b };          // Lexicographic
    ($a:expr > $b:expr) => { $a > $b };
    
    // Numeric comparisons
    ($a:expr -eq $b:expr) => { num_equals($a, $b) };
    ($a:expr -ne $b:expr) => { !num_equals($a, $b) };
    ($a:expr -lt $b:expr) => { num_less_than($a, $b) };
    ($a:expr -le $b:expr) => { num_less_than($a, $b) || num_equals($a, $b) };
    ($a:expr -gt $b:expr) => { num_greater_than($a, $b) };
    ($a:expr -ge $b:expr) => { num_greater_than($a, $b) || num_equals($a, $b) };
    
    // Logical operations
    ($a:expr && $b:expr) => { $a && $b };
    ($a:expr || $b:expr) => { $a || $b };
    (! $expr:expr) => { !$expr };
}

// Usage examples
if test!(-f "config.txt") && test!(-r "config.txt") {
    load_config!("config.txt");
}

if test!("VERSION" =~ r"^\d+\.\d+\.\d+$") {
    info!("Valid version format");
}

if test!(param!("COUNT") -gt "100") {
    warn!("Count is very high: $COUNT");
}
```

## 3. Command Substitution & Streaming

Building on your existing stream system with command substitution:

```rust
// Direct shell execution macro
macro_rules! shell {
    ($cmd:expr) => {
        std::process::Command::new("sh")
            .arg("-c")
            .arg(&var!($cmd).expand())  // Allow variable expansion
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).trim_end().to_string())
            .unwrap_or_else(|_| {
                error!("Shell command failed: {}", $cmd);
                std::process::exit(1);
            })
    };
    ($cmd:expr, silent) => {
        std::process::Command::new("sh")
            .arg("-c") 
            .arg(&var!($cmd).expand())
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    };
}

// Enhanced cmd! macro for streaming
macro_rules! cmd {
    // Basic command streaming (existing)
    ($command:expr) => {
        Stream::from_cmd($command)
    };
    
    // Command with variable capture (bash $(command) equivalent)
    ($command:expr => $var:expr) => {
        {
            let output = shell!($command);
            set_var($var, &output);
            output
        }
    };
    
    // Command with error handling
    ($command:expr, on_error: $handler:expr) => {
        {
            let result = std::process::Command::new("sh")
                .arg("-c")
                .arg(&var!($command).expand())
                .output();
            
            match result {
                Ok(output) if output.status.success() => {
                    Stream::from_string(&String::from_utf8_lossy(&output.stdout))
                },
                _ => {
                    $handler;
                    Stream::new() // Empty stream
                }
            }
        }
    };
    
    // Background execution
    ($command:expr, background) => {
        {
            let child = std::process::Command::new("sh")
                .arg("-c")
                .arg(&var!($command).expand())
                .spawn()
                .expect("Failed to spawn background process");
            child.id()
        }
    };
}

// Usage examples
let git_hash = shell!("git rev-parse HEAD");
set_var("GIT_HASH", &git_hash);

// Bash: VAR=$(command) becomes:
cmd!("git status --porcelain" => "GIT_STATUS");
echo!("Git status: $GIT_STATUS");

// Streaming with error handling
cmd!("risky_command", on_error: warn!("Command failed, continuing..."))
    .grep("important")
    .to_file("results.txt");

// Background processes
let pid = cmd!("long_running_task.sh", background);
set_var("TASK_PID", &pid.to_string());
```

## 4. Enhanced Streaming - Files, Variables, and Pipes

Complete streaming system with multiple input/output sources:

```rust
// Enhanced Stream with multiple sources and destinations
impl Stream {
    // Input sources
    pub fn from_var(var_name: &str) -> Self {
        let content = get_var(var_name);
        Stream::from_string(&content)
    }
    
    pub fn from_files(paths: &[&str]) -> Self {
        let mut content = String::new();
        for path in paths {
            let expanded_path = var!(path).expand();
            let file_content = read_file(&expanded_path);
            if !content.is_empty() {
                content.push('\n');
            }
            content.push_str(&file_content);
        }
        Stream::from_string(&content)
    }
    
    // Output destinations  
    pub fn to_var(self, var_name: &str) -> Self {
        let content = self.to_string();
        set_var(var_name, &content);
        self
    }
    
    pub fn append_to_file(self, path: &str) -> Self {
        let content = self.to_string();
        let expanded_path = var!(path).expand();
        append_file(&expanded_path, &content);
        self
    }
    
    pub fn to_both(self, file: &str, var: &str) -> Self {
        let content = self.to_string();
        let expanded_path = var!(file).expand();
        write_file(&expanded_path, &content);
        set_var(var, &content);
        self
    }
    
    // Advanced piping
    pub fn pipe_to_cmd(self, command: &str) -> Stream {
        let input = self.to_string();
        let expanded_cmd = var!(command).expand();
        
        let mut child = std::process::Command::new("sh")
            .arg("-c")
            .arg(&expanded_cmd)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn command");
            
        if let Some(stdin) = child.stdin.as_mut() {
            use std::io::Write;
            stdin.write_all(input.as_bytes()).expect("Failed to write to stdin");
        }
        
        let output = child.wait_with_output().expect("Failed to read stdout");
        Stream::from_string(&String::from_utf8_lossy(&output.stdout))
    }
}

// Enhanced macros for streaming
macro_rules! stream {
    (var: $var:expr) => { Stream::from_var($var) };
    (files: $($path:expr),+) => { Stream::from_files(&[$($path),+]) };
    (cmd: $command:expr) => { Stream::from_cmd($command) };
    (string: $content:expr) => { Stream::from_string($content) };
}

// Usage examples combining everything
stream!(files: "data1.txt", "data2.txt")
    .grep(&param!("SEARCH_PATTERN", default: "error"))
    .sed(&param!("OLD_TEXT"), &param!("NEW_TEXT"))
    .to_both(&param!("OUTPUT_FILE"), "PROCESSED_DATA");

// Complex pipeline with command substitution
stream!(cmd: "find $HOME -name '*.log'")
    .head(100)
    .each(|file| {
        let size = shell!("wc -l < {}", file);
        if test!(size -gt "1000") {
            echo!("Large log file: {blue}$file{reset} ({yellow}$size{reset} lines)");
        }
    });

// Stream to variable then process
stream!(cmd: "ps aux")
    .grep(&get_var("PROCESS_NAME"))
    .cut(2, " ")  // Get PID column
    .to_var("PIDS")
    .each(|pid| {
        if shell!(format!("kill -0 {}", pid), silent) {
            info!("Process $pid is running");
        }
    });
```

## 5. Error Handling Strategy

Based on your preference for fail-fast but with degrees of recovery:

```rust
// Error severity levels
pub enum ErrorLevel {
    Recoverable,    // Log warning, continue
    UserError,      // Show error, return error code
    SystemError,    // Show error, exit with code
    Fatal,          // Show error, exit immediately
}

// Error handling macros
macro_rules! try_or {
    ($operation:expr, recoverable: $fallback:expr) => {
        match $operation {
            Ok(result) => result,
            Err(e) => {
                warn!("Operation failed (recoverable): {}", e);
                $fallback
            }
        }
    };
    
    ($operation:expr, user_error: $message:expr) => {
        match $operation {
            Ok(result) => result,
            Err(e) => {
                error!("{}: {}", $message, e);
                return 1;
            }
        }
    };
    
    ($operation:expr, system_error: $message:expr) => {
        match $operation {
            Ok(result) => result,
            Err(e) => {
                error!("{}: {}", $message, e);
                std::process::exit(1);
            }
        }
    };
}

// File operations with appropriate error levels
pub fn mkdir_p(path: &str) -> bool {
    let expanded = var!(path).expand();
    std::fs::create_dir_all(&expanded)
        .map(|_| true)
        .unwrap_or_else(|e| {
            error!("Failed to create directory {}: {}", expanded, e);
            false  // Recoverable - caller can decide
        })
}

pub fn rm_rf(path: &str) -> bool {
    let expanded = var!(path).expand();
    // Warn for safety but don't fail
    if test!(-e &expanded) {
        warn!("Removing: {}", expanded);
        std::fs::remove_dir_all(&expanded)
            .or_else(|_| std::fs::remove_file(&expanded))
            .map(|_| true)
            .unwrap_or_else(|e| {
                error!("Failed to remove {}: {}", expanded, e);
                false
            })
    } else {
        trace!("Path doesn't exist (ok): {}", expanded);
        true
    }
}

// Usage
if !mkdir_p("$OUTPUT_DIR") {
    error!("Cannot create output directory, aborting");
    return 1;
}

// Or with try_or macro
let content = try_or!(
    read_file("config.txt"), 
    recoverable: "# Default config\nDEBUG=false\n".to_string()
);
```

This gives you the full bash-to-RSB translation capability while maintaining the string-first philosophy and providing flexible error handling!
