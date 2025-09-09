use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Global context - like shell environment variables
pub struct Context {
    vars: HashMap<String, String>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            vars: HashMap::new(),
        }
    }

    pub fn set<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) {
        self.vars.insert(key.into(), value.into());
    }

    pub fn get(&self, key: &str) -> String {
        self.vars.get(key).cloned().unwrap_or_default()
    }

    pub fn has(&self, key: &str) -> bool {
        self.vars.contains_key(key)
    }

    pub fn expand(&self, text: &str) -> String {
        use regex::Regex;
        
        let mut result = text.to_string();
        
        // Handle ${VAR} syntax
        let braced_re = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap();
        result = braced_re.replace_all(&result, |caps: &regex::Captures| {
            let var_name = &caps[1];
            self.vars.get(var_name).cloned().unwrap_or_default()
        }).to_string();
        
        // Handle $VAR syntax (word boundaries)
        let simple_re = Regex::new(r"\$([A-Za-z_][A-Za-z0-9_]*)").unwrap();
        result = simple_re.replace_all(&result, |caps: &regex::Captures| {
            let var_name = &caps[1];
            self.vars.get(var_name).cloned().unwrap_or_default()
        }).to_string();
        
        result
    }
}

// Thread-safe global context
lazy_static::lazy_static! {
    pub static ref CTX: Arc<Mutex<Context>> = Arc::new(Mutex::new(Context::new()));
    static ref FUNCTION_REGISTRY: Arc<Mutex<HashMap<String, String>>> = 
        Arc::new(Mutex::new(HashMap::new()));
    static ref CALL_STACK: Arc<Mutex<Vec<CallFrame>>> = 
        Arc::new(Mutex::new(Vec::new()));
    static ref COLORS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("red", "\x1b[31m");
        m.insert("green", "\x1b[32m");
        m.insert("yellow", "\x1b[33m");
        m.insert("blue", "\x1b[34m");
        m.insert("grey", "\x1b[90m");
        m.insert("cyan", "\x1b[36m");
        m.insert("magenta", "\x1b[35m");
        m.insert("reset", "\x1b[0m");
        m.insert("bold", "\x1b[1m");
        m
    };
}

// Enhanced argument handling with flag extraction
pub struct Args {
    args: Vec<String>,
    processed: std::collections::HashSet<usize>,
}

impl Args {
    pub fn new(args: &[String]) -> Self {
        Args { 
            args: args.to_vec(),
            processed: std::collections::HashSet::new(),
        }
    }

    // Get positional arg like $1, $2 in bash (1-indexed like bash)
    pub fn get(&self, n: usize) -> &str {
        if n == 0 {
            // $0 returns script name
            &get_var("SCRIPT_NAME")
        } else {
            // Skip processed args when counting positional args
            let mut pos = 0;
            for (i, _) in self.args.iter().enumerate() {
                if !self.processed.contains(&i) {
                    if pos == n - 1 {
                        return &self.args[i];
                    }
                    pos += 1;
                }
            }
            ""
        }
    }

    // Get with default value
    pub fn get_or(&self, n: usize, default: &str) -> &str {
        let val = self.get(n);
        if val.is_empty() { default } else { val }
    }

    // Check for flags and optionally pop them
    pub fn has(&self, flag: &str) -> bool {
        self.args.iter().any(|arg| arg == flag)
    }

    // Pop flag from args (marks as processed)
    pub fn has_pop(&mut self, flag: &str) -> bool {
        if let Some(pos) = self.args.iter().position(|arg| arg == flag) {
            self.processed.insert(pos);
            true
        } else {
            false
        }
    }

    // Get flag value: --flag value or --flag=value
    pub fn has_val(&mut self, flag: &str) -> Option<String> {
        // Check for --flag=value format
        for (i, arg) in self.args.iter().enumerate() {
            if arg.starts_with(&format!("{}=", flag)) {
                self.processed.insert(i);
                return Some(arg.split('=').nth(1).unwrap_or("").to_string());
            }
        }
        
        // Check for --flag value format
        if let Some(pos) = self.args.iter().position(|arg| arg == flag) {
            if pos + 1 < self.args.len() {
                self.processed.insert(pos);
                self.processed.insert(pos + 1);
                return Some(self.args[pos + 1].clone());
            }
        }
        
        None
    }

    // Parse key:value or key=value arguments
    pub fn get_kv(&mut self, key: &str) -> Option<String> {
        for (i, arg) in self.args.iter().enumerate() {
            // Check key:value format
            if arg.starts_with(&format!("{}:", key)) {
                self.processed.insert(i);
                return Some(arg.split(':').nth(1).unwrap_or("").to_string());
            }
            // Check key=value format  
            if arg.starts_with(&format!("{}=", key)) {
                self.processed.insert(i);
                return Some(arg.split('=').nth(1).unwrap_or("").to_string());
            }
        }
        None
    }

    // Parse array arguments: key=1,2,3 or key:a,b,c
    pub fn get_array(&mut self, key: &str) -> Option<Vec<String>> {
        if let Some(value) = self.get_kv(key) {
            return Some(value.split(',').map(|s| s.trim().to_string()).collect());
        }
        None
    }

    // Get all unprocessed args
    pub fn remaining(&self) -> Vec<String> {
        self.args.iter()
            .enumerate()
            .filter(|(i, _)| !self.processed.contains(i))
            .map(|(_, arg)| arg.clone())
            .collect()
    }

    // Get all args
    pub fn all(&self) -> &[String] {
        &self.args
    }

    // Join unprocessed args with separator
    pub fn join(&self, sep: &str) -> String {
        self.remaining().join(sep)
    }

    // Count of unprocessed args
    pub fn len(&self) -> usize {
        self.remaining().len()
    }

    // Replace placeholders in string with positional args AND context vars
    pub fn expand(&self, template: &str) -> String {
        let mut result = template.to_string();
        
        // First replace positional args $1, $2, etc.
        let remaining = self.remaining();
        for (i, arg) in remaining.iter().enumerate() {
            let placeholder = format!("${}", i + 1);
            result = result.replace(&placeholder, arg);
        }
        
        // Replace $@ with all unprocessed args
        result = result.replace("$@", &self.join(" "));
        
        // Replace $# with unprocessed arg count
        result = result.replace("$#", &self.len().to_string());
        
        // Then expand context variables using global context
        result = expand_vars(&result);
        
        result
    }
}

// RSB Stderr Implementation with BashFX-style color support
pub fn stderr_with_color(level: &str, message: &str) {
    if should_print_level(level) {
        let colored_msg = expand_colors(&expand_vars(message));
        eprintln!("{}", colored_msg);
    }
}

pub fn expand_colors(text: &str) -> String {
    let mut result = text.to_string();
    for (name, code) in COLORS.iter() {
        result = result.replace(&format!("${{{}}}", name), code);
    }
    result
}

pub fn should_print_level(level: &str) -> bool {
    if has_var("QUIET_MODE") && !["error", "fatal"].contains(&level) {
        return false;
    }
    
    match level {
        "trace" | "think" => has_var("DEBUG_MODE") && has_var("TRACE_MODE"),
        "info" | "warn" | "okay" => has_var("DEBUG_MODE") || has_var("DEV_MODE"),
        "error" | "fatal" => true,
        _ => has_var("DEBUG_MODE")
    }
}

// BashFX-style stderr macros
#[macro_export]
macro_rules! info {
    ($msg:expr) => {
        stderr_with_color("info", $msg);
    };
    ($msg:expr, $($args:expr),*) => {
        let expanded = var!($msg).expand();
        let formatted = format!(&expanded, $($args),*);
        stderr_with_color("info", &formatted);
    };
}

#[macro_export]
macro_rules! error {
    ($msg:expr) => {
        stderr_with_color("error", $msg);
    };
    ($msg:expr, $($args:expr),*) => {
        let expanded = var!($msg).expand();
        let formatted = format!(&expanded, $($args),*);
        stderr_with_color("error", &formatted);
    };
}

#[macro_export]
macro_rules! okay {
    ($msg:expr) => {
        stderr_with_color("okay", $msg);
    };
    ($msg:expr, $($args:expr),*) => {
        let expanded = var!($msg).expand();
        let formatted = format!(&expanded, $($args),*);
        stderr_with_color("okay", &formatted);
    };
}

#[macro_export]
macro_rules! warn {
    ($msg:expr) => {
        stderr_with_color("warn", $msg);
    };
    ($msg:expr, $($args:expr),*) => {
        let expanded = var!($msg).expand();
        let formatted = format!(&expanded, $($args),*);
        stderr_with_color("warn", &formatted);
    };
}

#[macro_export]
macro_rules! trace {
    ($msg:expr) => {
        stderr_with_color("trace", $msg);
    };
    ($msg:expr, $($args:expr),*) => {
        let expanded = var!($msg).expand();
        let formatted = format!(&expanded, $($args),*);
        stderr_with_color("trace", &formatted);
    };
}

// Debug helpers for complex variables
pub fn debug_var<T: std::fmt::Debug>(name: &str, value: &T) {
    if should_print_level("debug") {
        stderr_with_color("debug", &format!("{grey}DEBUG:{reset} {} = {:#?}", name, value));
    }
}

pub fn trace_vars(vars: &[(&str, &dyn std::fmt::Debug)]) {
    if should_print_level("trace") {
        for (name, value) in vars {
            stderr_with_color("trace", &format!("{grey}TRACE:{reset} {} = {:#?}", name, value));
        }
    }
}

// Type checking and validation functions
pub fn is_file(path: &str) -> bool {
    let expanded = var!(path).expand();
    std::path::Path::new(&expanded).is_file()
}

pub fn is_dir(path: &str) -> bool {
    let expanded = var!(path).expand();
    std::path::Path::new(&expanded).is_dir()
}

pub fn is_entity(path: &str) -> bool {
    let expanded = var!(path).expand();
    std::path::Path::new(&expanded).exists()
}

pub fn is_link(path: &str) -> bool {
    let expanded = var!(path).expand();
    std::path::Path::new(&expanded).is_symlink()
}

pub fn is_string(value: &str) -> bool {
    !value.is_empty()
}

pub fn is_numeric(value: &str) -> bool {
    value.parse::<f64>().is_ok()
}

pub fn is_empty(value: &str) -> bool {
    value.trim().is_empty()
}

pub fn is_name(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

pub fn is_command(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn is_function(name: &str) -> bool {
    FUNCTION_REGISTRY.lock().unwrap().contains_key(name)
}

// Call stack management for debugging and introspection
#[derive(Debug, Clone)]
pub struct CallFrame {
    pub function: String,
    pub args: Vec<String>,
    pub timestamp: std::time::SystemTime,
    pub context_snapshot: HashMap<String, String>,
}

pub fn push_call(function: &str, args: &[String]) {
    let frame = CallFrame {
        function: function.to_string(),
        args: args.to_vec(),
        timestamp: std::time::SystemTime::now(),
        context_snapshot: CTX.lock().unwrap().vars.clone(),
    };
    CALL_STACK.lock().unwrap().push(frame);
}

pub fn pop_call() -> Option<CallFrame> {
    CALL_STACK.lock().unwrap().pop()
}

pub fn get_call_stack() -> Vec<CallFrame> {
    CALL_STACK.lock().unwrap().clone()
}

pub fn register_function(name: &str, description: &str) {
    FUNCTION_REGISTRY.lock().unwrap().insert(name.to_string(), description.to_string());
}

pub fn list_functions() -> Vec<(String, String)> {
    FUNCTION_REGISTRY.lock().unwrap()
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

// Var construct for easy variable expansion
pub struct Var(String);

impl Var {
    pub fn new(text: &str) -> Self {
        Var(expand_vars(text))
    }
    
    pub fn expand(&self) -> String {
        self.0.clone()
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Var {
    fn from(text: &str) -> Self {
        Var::new(text)
    }
}

impl From<String> for Var {
    fn from(text: String) -> Self {
        Var::new(&text)
    }
}

// Macro for easy variable creation
#[macro_export]
macro_rules! var {
    ($text:expr) => {
        Var::new($text)
    };
}

// Echo macro for bash-like output with variable expansion
#[macro_export]
macro_rules! echo {
    ($text:expr) => {
        println!("{}", var!($text).expand());
    };
    ($text:expr, $($args:expr),*) => {
        println!("{}", format!($text, $($args),*));
    };
}

// BashFX Thisness Pattern for library context
pub fn set_this_context(namespace: &str, root_dir: &str, config_file: &str) {
    set_var("THIS_NAMESPACE", namespace);
    set_var("THIS_ROOT", root_dir);
    set_var("THIS_CONFIG", config_file);
    set_var("THIS_BIN", &var!("$RSB_BIN/$THIS_NAMESPACE").expand());
    set_var("THIS_LIB", &var!("$RSB_LIB/$THIS_NAMESPACE").expand());
}

// XDG+ setup with RSB/ODX namespacing
pub fn setup_xdg_paths() {
    set_var("XDG_HOME", "$HOME/.local");
    set_var("XDG_LIB", "$XDG_HOME/lib");
    set_var("XDG_ETC", "$XDG_HOME/etc");
    set_var("XDG_BIN", "$XDG_HOME/bin");
    set_var("XDG_DATA", "$XDG_HOME/data");
    set_var("XDG_TMP", "$HOME/.cache/tmp");
}

pub fn setup_rsb_paths() {
    set_var("RSB_LIB", "$XDG_LIB/rsb");
    set_var("RSB_BIN", "$XDG_BIN/rsb");
    set_var("RSB_ETC", "$XDG_ETC/rsb");
    set_var("RSB_DATA", "$XDG_DATA/rsb");
    set_var("ODX_LIB", "$XDG_LIB/odx");
    set_var("ODX_BIN", "$XDG_BIN/odx");
}

pub fn setup_standard_modes() {
    // BashFX-style mode variables from environment
    if std::env::var("DEBUG").is_ok() { set_var("DEBUG_MODE", "true"); }
    if std::env::var("DEV").is_ok() { set_var("DEV_MODE", "true"); }
    if std::env::var("QUIET").is_ok() { set_var("QUIET_MODE", "true"); }
    if std::env::var("TRACE").is_ok() { set_var("TRACE_MODE", "true"); }
}

pub fn setup_script_awareness(args: &[String]) {
    let script_path = &args[0];
    let script_name = std::path::Path::new(script_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("script");
    let script_dir = std::path::Path::new(script_path)
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or(".");
    
    set_var("SCRIPT_NAME", script_name);
    set_var("SCRIPT_PATH", script_path);
    set_var("SCRIPT_DIR", script_dir);
}

// Standard RSB bootstrap function
pub fn rsb_bootstrap(args: &[String]) {
    // Front-load all environment variables
    for (key, value) in std::env::vars() {
        set_var(&key, &value);
    }
    
    setup_xdg_paths();
    setup_rsb_paths();
    setup_standard_modes();
    setup_script_awareness(args);
}

// Sentinel-based linking for rewindable operations
pub fn link_with_sentinel(target_file: &str, line_to_add: &str, sentinel: &str) {
    let expanded_target = var!(target_file).expand();
    let content = read_file(&expanded_target);
    let sentinel_line = format!("{} # {}", line_to_add, sentinel);
    
    // Check if already linked
    if !content.contains(sentinel) {
        let new_content = if content.trim().is_empty() {
            sentinel_line
        } else {
            format!("{}\n{}", content, sentinel_line)
        };
        write_file(&expanded_target, &new_content);
        info!("Linked: {grey}$sentinel_line{reset}");
    } else {
        trace!("Already linked: {grey}$sentinel{reset}");
    }
}

pub fn unlink_with_sentinel(target_file: &str, sentinel: &str) {
    let expanded_target = var!(target_file).expand();
    let content = read_file(&expanded_target);
    let filtered = content
        .lines()
        .filter(|line| !line.contains(&format!("# {}", sentinel)))
        .collect::<Vec<_>>()
        .join("\n");
    
    write_file(&expanded_target, &filtered);
    info!("Unlinked: {grey}$sentinel{reset}");
}

// Enhanced dispatch macros with function registration and call stack
#[macro_export]
macro_rules! rsb_dispatch {
    ($args:expr, {
        $($cmd:literal => $handler:ident),*
    }) => {
        let command = $args.get(1).unwrap_or("help");
        let cmd_args = Args::new(&$args[2..]);
        
        // Register functions for introspection
        $(register_function($cmd, stringify!($handler));)*
        
        match command {
            $($cmd => {
                push_call($cmd, cmd_args.all());
                let result = $handler(cmd_args);
                pop_call();
                std::process::exit(result);
            },)*
            "help" | "--help" | "-h" => {
                show_help();
                std::process::exit(0);
            },
            "inspect" => {
                show_functions();
                std::process::exit(0);
            },
            "stack" => {
                show_call_stack();
                std::process::exit(0);
            },
            _ => {
                error!("Unknown command: {red}$command{reset}");
                show_help();
                std::process::exit(1);
            }
        }
    }
}

// Pre-context dispatcher for bootstrap commands
#[macro_export]
macro_rules! rsb_pre_dispatch {
    ($args:expr, {
        $($cmd:literal => $handler:ident),*
    }) => {
        {
            let command = $args.get(1).unwrap_or("help");
            let cmd_args = Args::new(&$args[2..]);
            
            match command {
                $($cmd => {
                    push_call($cmd, cmd_args.all());
                    let result = $handler(cmd_args);
                    pop_call();
                    std::process::exit(result);
                },)*
                _ => {
                    // If not a pre-command, continue to main dispatch
                    false
                }
            }
        }
    }
}

// Built-in inspection functions
fn show_help() {
    echo!("{bold}{blue}$SCRIPT_NAME{reset} - RSB Application");
    echo!("");
    echo!("Available commands:");
    for (name, desc) in list_functions() {
        println!("  {:<15} {}", name, desc);
    }
    echo!("");
    echo!("Built-in commands:");
    echo!("  {cyan}help{reset}            Show this help");
    echo!("  {cyan}inspect{reset}         List all available functions");
    echo!("  {cyan}stack{reset}           Show current call stack");
}

fn show_functions() {
    echo!("{bold}Available functions:{reset}");
    for (name, desc) in list_functions() {
        println!("  {:<20} {}", name, desc);
    }
}

fn show_call_stack() {
    let stack = get_call_stack();
    if stack.is_empty() {
        echo!("Call stack is empty");
        return;
    }
    
    echo!("{bold}Call stack (most recent first):{reset}");
    for (i, frame) in stack.iter().rev().enumerate() {
        let elapsed = frame.timestamp.elapsed()
            .map(|d| format!("{}ms", d.as_millis()))
            .unwrap_or("?".to_string());
        
        println!("  {}: {yellow}{}{reset} {} ({grey}{}{})", 
            i, 
            frame.function, 
            frame.args.join(" "),
            elapsed,
            "{reset}"
        );
    }
}

// Validation macro for argument checking
#[macro_export]
macro_rules! validate {
    ($condition:expr, $message:expr) => {
        if !$condition {
            error!("Validation failed: {}", $message);
            std::process::exit(1);
        }
    };
    ($condition:expr, $message:expr, $code:expr) => {
        if !$condition {
            error!("Validation failed: {}", $message);
            std::process::exit($code);
        }
    };
}

// Convenience macros for type checking with validation
#[macro_export]
macro_rules! require_file {
    ($path:expr) => {
        validate!(is_file($path), format!("File does not exist: {}", $path));
    };
}

#[macro_export]
macro_rules! require_dir {
    ($path:expr) => {
        validate!(is_dir($path), format!("Directory does not exist: {}", $path));
    };
}

#[macro_export]
macro_rules! require_command {
    ($cmd:expr) => {
        validate!(is_command($cmd), format!("Command not found: {}", $cmd));
    };
}

#[macro_export]
macro_rules! require_var {
    ($var:expr) => {
        validate!(has_var($var), format!("Required variable not set: {}", $var));
    };
}

// Configuration file handling
pub fn load_config_file(path: &str) {
    let expanded_path = var!(path).expand();
    if !std::path::Path::new(&expanded_path).exists() {
        trace!("Config file not found: {grey}$expanded_path{reset}");
        return;
    }

    let content = read_file(&expanded_path);
    parse_config_content(&content);
    trace!("Loaded config: {grey}$expanded_path{reset}");
}

pub fn parse_config_content(content: &str) {
    for line in content.lines() {
        let line = line.trim();
        
        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Handle simple key=value pairs
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            
            // Remove quotes if present
            let value = if (value.starts_with('"') && value.ends_with('"')) ||
                          (value.starts_with('\'') && value.ends_with('\'')) {
                &value[1..value.len()-1]
            } else {
                value
            };

            // Handle bash-style arrays: ARRAY=(item1 item2 item3)
            if value.starts_with('(') && value.ends_with(')') {
                let array_content = &value[1..value.len()-1];
                let items: Vec<&str> = array_content.split_whitespace().collect();
                
                // Store array length
                set_var(&format!("{}_LENGTH", key), &items.len().to_string());
                
                // Store each item with index
                for (i, item) in items.iter().enumerate() {
                    set_var(&format!("{}_{}", key, i), item);
                }
                
                // Store the whole array as space-separated string
                set_var(key, &items.join(" "));
            } else {
                // Regular variable
                set_var(key, value);
            }
        }
    }
}

pub fn save_config_file(path: &str, keys: &[&str]) {
    let expanded_path = var!(path).expand();
    let mut content = String::new();
    content.push_str("# RSB Configuration File\n");
    content.push_str(&format!("# Generated on {}\n\n", 
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    ));

    for key in keys {
        if has_var(key) {
            let value = get_var(key);
            
            // Check if this looks like an array (has _LENGTH suffix var)
            let length_key = format!("{}_LENGTH", key);
            if has_var(&length_key) {
                let length: usize = get_var(&length_key).parse().unwrap_or(0);
                let mut items = Vec::new();
                
                for i in 0..length {
                    let item_key = format!("{}_{}", key, i);
                    if has_var(&item_key) {
                        items.push(get_var(&item_key));
                    }
                }
                
                content.push_str(&format!("{}=({})\n", key, items.join(" ")));
            } else {
                // Quote values that contain spaces
                if value.contains(' ') {
                    content.push_str(&format!("{}=\"{}\"\n", key, value));
                } else {
                    content.push_str(&format!("{}={}\n", key, value));
                }
            }
        }
    }

    write_file(&expanded_path, &content);
}

// Array helpers
pub fn set_array(key: &str, items: &[&str]) {
    // Store array length
    set_var(&format!("{}_LENGTH", key), &items.len().to_string());
    
    // Store each item with index
    for (i, item) in items.iter().enumerate() {
        set_var(&format!("{}_{}", key, i), item);
    }
    
    // Store the whole array as space-separated string
    set_var(key, &items.join(" "));
}

pub fn get_array(key: &str) -> Vec<String> {
    let length_key = format!("{}_LENGTH", key);
    if !has_var(&length_key) {
        // Try to parse as space-separated string
        let value = get_var(key);
        if value.is_empty() {
            return Vec::new();
        }
        return value.split_whitespace().map(|s| s.to_string()).collect();
    }

    let length: usize = get_var(&length_key).parse().unwrap_or(0);
    let mut items = Vec::new();
    
    for i in 0..length {
        let item_key = format!("{}_{}", key, i);
        if has_var(&item_key) {
            items.push(get_var(&item_key));
        }
    }
    
    items
}

pub fn push_array(key: &str, item: &str) {
    let mut items = get_array(key);
    items.push(item.to_string());
    let item_strs: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
    set_array(key, &item_strs);
}

// String utilities for common operations
pub trait StringExt {
    fn cut(&self, field: usize, delimiter: &str) -> String;
    fn grep(&self, pattern: &str) -> Vec<String>;
    fn sed(&self, from: &str, to: &str) -> String;
    fn head(&self, n: usize) -> Vec<String>;
    fn tail(&self, n: usize) -> Vec<String>;
    fn trim_lines(&self) -> String;
    fn sort(&self) -> Vec<String>;
    fn uniq(&self) -> Vec<String>;
    fn wc_l(&self) -> usize;
    fn cat(&self, other: &str) -> String;
    fn tee(&self, file: &str) -> String;
    fn filter<F>(&self, predicate: F) -> Vec<String>
    where
        F: Fn(&str) -> bool;
}

impl StringExt for String {
    fn cut(&self, field: usize, delimiter: &str) -> String {
        self.lines()
            .filter_map(|line| line.split(delimiter).nth(field - 1)) // 1-indexed like bash
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn grep(&self, pattern: &str) -> Vec<String> {
        self.lines()
            .filter(|line| line.contains(pattern))
            .map(|s| s.to_string())
            .collect()
    }

    fn sed(&self, from: &str, to: &str) -> String {
        self.replace(from, to)
    }

    fn head(&self, n: usize) -> Vec<String> {
        self.lines().take(n).map(|s| s.to_string()).collect()
    }

    fn tail(&self, n: usize) -> Vec<String> {
        let lines: Vec<&str> = self.lines().collect();
        lines.iter()
            .rev()
            .take(n)
            .rev()
            .map(|s| s.to_string())
            .collect()
    }

    fn trim_lines(&self) -> String {
        self.lines()
            .map(|line| line.trim())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn sort(&self) -> Vec<String> {
        let mut lines: Vec<String> = self.lines().map(|s| s.to_string()).collect();
        lines.sort();
        lines
    }

    fn uniq(&self) -> Vec<String> {
        use std::collections::LinkedHashSet;
        self.lines()
            .map(|s| s.to_string())
            .collect::<LinkedHashSet<_>>()
            .into_iter()
            .collect()
    }

    fn wc_l(&self) -> usize {
        self.lines().count()
    }

    fn cat(&self, other: &str) -> String {
        format!("{}\n{}", self, other)
    }

    fn tee(&self, file: &str) -> String {
        write_file(file, self);
        self.clone()
    }

    fn filter<F>(&self, predicate: F) -> Vec<String>
    where
        F: Fn(&str) -> bool,
    {
        self.lines()
            .filter(|line| predicate(line))
            .map(|s| s.to_string())
            .collect()
    }
}

// Stream operations for chainable processing
pub struct Stream {
    lines: Vec<String>,
}

impl Stream {
    pub fn new() -> Self {
        Stream { lines: Vec::new() }
    }

    pub fn from_string(content: &str) -> Self {
        Stream {
            lines: content.lines().map(|s| s.to_string()).collect(),
        }
    }

    pub fn from_file(path: &str) -> Self {
        let expanded_path = var!(path).expand();
        let content = read_file(&expanded_path);
        Stream::from_string(&content)
    }

    pub fn from_cmd(cmd: &str) -> Self {
        let expanded_cmd = var!(cmd).expand();
        let output = run_cmd(&expanded_cmd);
        Stream::from_string(&output)
    }

    // Chainable operations
    pub fn grep(mut self, pattern: &str) -> Self {
        self.lines.retain(|line| line.contains(pattern));
        self
    }

    pub fn sed(mut self, from: &str, to: &str) -> Self {
        self.lines = self.lines.iter()
            .map(|line| line.replace(from, to))
            .collect();
        self
    }

    pub fn cut(mut self, field: usize, delimiter: &str) -> Self {
        self.lines = self.lines.iter()
            .filter_map(|line| line.split(delimiter).nth(field - 1)) // 1-indexed
            .map(|s| s.to_string())
            .collect();
        self
    }

    pub fn head(mut self, n: usize) -> Self {
        self.lines.truncate(n);
        self
    }

    pub fn tail(mut self, n: usize) -> Self {
        let len = self.lines.len();
        if len > n {
            self.lines = self.lines.into_iter().skip(len - n).collect();
        }
        self
    }

    pub fn sort(mut self) -> Self {
        self.lines.sort();
        self
    }

    pub fn uniq(mut self) -> Self {
        use std::collections::LinkedHashSet;
        self.lines = self.lines.into_iter()
            .collect::<LinkedHashSet<_>>()
            .into_iter()
            .collect();
        self
    }

    pub fn filter<F>(mut self, predicate: F) -> Self
    where
        F: Fn(&str) -> bool,
    {
        self.lines.retain(|line| predicate(line));
        self
    }

    pub fn map<F>(mut self, mapper: F) -> Self
    where
        F: Fn(&str) -> String,
    {
        self.lines = self.lines.iter()
            .map(|line| mapper(line))
            .collect();
        self
    }

    // Output operations
    pub fn to_string(self) -> String {
        self.lines.join("\n")
    }

    pub fn to_vec(self) -> Vec<String> {
        self.lines
    }

    pub fn to_file(self, path: &str) -> Self {
        let expanded_path = var!(path).expand();
        write_file(&expanded_path, &self.to_string());
        self
    }

    pub fn tee(self, path: &str) -> Self {
        let expanded_path = var!(path).expand();
        write_file(&expanded_path, &self.to_string());
        self
    }

    pub fn each<F>(self, action: F) -> Self
    where
        F: Fn(&str),
    {
        for line in &self.lines {
            action(line);
        }
        self
    }

    pub fn count(&self) -> usize {
        self.lines.len()
    }

    pub fn first(&self) -> Option<&String> {
        self.lines.first()
    }

    pub fn last(&self) -> Option<&String> {
        self.lines.last()
    }
}

// Pipe macros for stream operations
#[macro_export]
macro_rules! pipe {
    ($input:expr) => {
        Stream::from_string(&$input.to_string())
    };
}

// File pipe operations
#[macro_export]
macro_rules! cat {
    ($file:expr) => {
        Stream::from_file($file)
    };
    ($($file:expr),+) => {
        {
            let mut result = String::new();
            $(
                if !result.is_empty() {
                    result.push('\n');
                }
                let expanded_file = var!($file).expand();
                result.push_str(&read_file(&expanded_file));
            )+
            Stream::from_string(&result)
        }
    };
}

// Command pipe
#[macro_export]
macro_rules! cmd {
    ($command:expr) => {
        Stream::from_cmd($command)
    };
}

// Array manipulation macros
#[macro_export]
macro_rules! array {
    ($name:expr, [$($item:expr),*]) => {
        {
            let items = vec![$($item),*];
            let item_strs: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
            set_array($name, &item_strs);
        }
    };
}

// Configuration loading macro
#[macro_export]
macro_rules! load_config {
    ($path:expr) => {
        load_config_file($path);
    };
    ($($path:expr),+) => {
        $(load_config_file($path);)+
    };
}

// Convenience functions for global context
pub fn set_var<K: Into<String>, V: Into<String>>(key: K, value: V) {
    CTX.lock().unwrap().set(key, value);
}

pub fn get_var(key: &str) -> String {
    CTX.lock().unwrap().get(key)
}

pub fn has_var(key: &str) -> bool {
    CTX.lock().unwrap().has(key)
}

pub fn unset_var(key: &str) {
    CTX.lock().unwrap().vars.remove(key);
}

pub fn expand_vars(text: &str) -> String {
    CTX.lock().unwrap().expand(text)
}

// File operations that return strings
pub fn read_file(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

pub fn write_file(path: &str, content: &str) {
    // Ensure directory exists
    if let Some(parent) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(parent).unwrap_or_else(|e| {
            error!("Failed to create directory {}: {}", parent.display(), e);
            std::process::exit(1);
        });
    }
    
    std::fs::write(path, content).unwrap_or_else(|e| {
        error!("Failed to write {}: {}", path, e);
        std::process::exit(1);
    });
}

pub fn run_cmd(cmd: &str) -> String {
    use std::process::Command;
    
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .expect("Failed to execute command");
    
    String::from_utf8_lossy(&output.stdout).to_string()
}

pub struct CmdResult {
    pub status: i32,
    pub output: String,
    pub error: String,
}

pub fn run_cmd_with_status(cmd: &str) -> CmdResult {
    use std::process::Command;
    
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .expect("Failed to execute command");
    
    CmdResult {
        status: output.status.code().unwrap_or(1),
        output: String::from_utf8_lossy(&output.stdout).to_string(),
        error: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}