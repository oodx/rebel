use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Represents a single frame in the function call stack for debugging and introspection.
#[derive(Debug, Clone)]
pub struct CallFrame {
    pub function: String,
    pub args: Vec<String>,
    pub timestamp: std::time::SystemTime,
    pub context_snapshot: HashMap<String, String>,
}

// The core struct for holding RSB's global state, mimicking shell environment variables.
pub struct Context {
    vars: HashMap<String, String>,
}

impl Context {
    // Creates a new, empty context.
    pub fn new() -> Self {
        Context {
            vars: HashMap::new(),
        }
    }

    // Sets a variable in the context.
    pub fn set<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) {
        self.vars.insert(key.into(), value.into());
    }

    // Gets a variable from the context, returning an empty string if not found.
    pub fn get(&self, key: &str) -> String {
        self.vars.get(key).cloned().unwrap_or_default()
    }

    // Checks if a variable exists in the context.
    pub fn has(&self, key: &str) -> bool {
        self.vars.contains_key(key)
    }

    // Expands variables in a string (e.g., "$HOME" or "${HOME}").
    pub fn expand(&self, text: &str) -> String {
        let mut result = text.to_string();
        let braced_re = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap();
        result = braced_re
            .replace_all(&result, |caps: &regex::Captures| {
                let var_name = &caps[1];
                self.vars.get(var_name).cloned().unwrap_or_default()
            })
            .to_string();
        let simple_re = Regex::new(r"\$([A-Za-z_][A-Za-z0-9_]*)").unwrap();
        result = simple_re
            .replace_all(&result, |caps: &regex::Captures| {
                let var_name = &caps[1];
                self.vars.get(var_name).cloned().unwrap_or_default()
            })
            .to_string();
        result
    }

    /// Returns a clone of all variables in the context.
    pub fn get_all_vars(&self) -> HashMap<String, String> {
        self.vars.clone()
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

lazy_static! {
    pub static ref CTX: Arc<Mutex<Context>> = Arc::new(Mutex::new(Context::new()));
    pub(crate) static ref FUNCTION_REGISTRY: Arc<Mutex<HashMap<String, String>>> =
        Arc::new(Mutex::new(HashMap::new()));
    pub(crate) static ref CALL_STACK: Arc<Mutex<Vec<CallFrame>>> =
        Arc::new(Mutex::new(Vec::new()));

    // Make COLORS and GLYPHS mutable for user configuration
    pub(crate) static ref COLORS: Arc<Mutex<HashMap<String, String>>> = {
        let mut m = HashMap::new();
        m.insert("red".to_string(), "\x1b[31m".to_string());
        m.insert("green".to_string(), "\x1b[32m".to_string());
        m.insert("yellow".to_string(), "\x1b[33m".to_string());
        m.insert("blue".to_string(), "\x1b[34m".to_string());
        m.insert("grey".to_string(), "\x1b[90m".to_string());
        m.insert("cyan".to_string(), "\x1b[36m".to_string());
        m.insert("magenta".to_string(), "\x1b[35m".to_string());
        m.insert("reset".to_string(), "\x1b[0m".to_string());
        m.insert("bold".to_string(), "\x1b[1m".to_string());
        Arc::new(Mutex::new(m))
    };
    pub(crate) static ref GLYPHS: Arc<Mutex<HashMap<String, String>>> = {
        let mut m = HashMap::new();
        m.insert("info".to_string(), "ℹ".to_string());
        m.insert("okay".to_string(), "✓".to_string());
        m.insert("warn".to_string(), "⚠".to_string());
        m.insert("error".to_string(), "✗".to_string());
        m.insert("fatal".to_string(), "💀".to_string());
        m.insert("debug".to_string(), "🔍".to_string());
        m.insert("trace".to_string(), "👁".to_string());
        Arc::new(Mutex::new(m))
    };
}

// Public API for interacting with the global context.
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

// --- Introspection and Call Stack Functions ---
pub fn register_function(name: &str, description: &str) {
    FUNCTION_REGISTRY
        .lock()
        .unwrap()
        .insert(name.to_string(), description.to_string());
}
pub fn list_functions() -> Vec<(String, String)> {
    let mut funcs: Vec<_> = FUNCTION_REGISTRY
        .lock()
        .unwrap()
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    funcs.sort_by(|a, b| a.0.cmp(&b.0));
    funcs
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
pub fn show_help() {
    println!(
        "{}",
        expand_vars(&format!(
            "{{bold}}{{blue}}{}{{reset}}\n\n{{bold}}USAGE:{{reset}}\n  {} <command> [options]\n\n{{bold}}COMMANDS:{{reset}}",
            get_var("SCRIPT_NAME"),
            get_var("SCRIPT_NAME")
        ))
    );
    for (name, desc) in list_functions() {
        println!("  {{cyan}}{:<15}{{reset}} {}", name, desc);
    }
    println!("\n{{bold}}BUILT-IN COMMANDS:{{reset}}");
    println!("  {{green}}{:<15}{{reset}} Show this help message", "help");
    println!("  {{green}}{:<15}{{reset}} List all available functions", "inspect");
    println!("  {{green}}{:<15}{{reset}} Show the current call stack", "stack");
}
pub fn show_functions() {
    println!("{{bold}}Available functions:{{reset}}");
    for (name, desc) in list_functions() {
        println!("  {{cyan}}{:<20}{{reset}} {}", name, desc);
    }
}
pub fn show_call_stack() {
    let stack = get_call_stack();
    if stack.is_empty() {
        println!("Call stack is empty");
        return;
    }
    println!("{{bold}}Call stack (most recent first):{{reset}}");
    for (i, frame) in stack.iter().rev().enumerate() {
        let elapsed = frame
            .timestamp
            .elapsed()
            .map(|d| format!("{}ms", d.as_millis()))
            .unwrap_or_else(|_| "?".to_string());
        println!(
            "  {}: {{yellow}}{}{{reset}} {} ({{grey}}{} ago{{reset}})",
            i,
            frame.function,
            frame.args.join(" "),
            elapsed
        );
    }
}

// --- Config File Functions ---
pub fn parse_config_content(content: &str) {
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            let value = value
                .strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .unwrap_or(value);
            let value = value
                .strip_prefix('\'')
                .and_then(|v| v.strip_suffix('\''))
                .unwrap_or(value);
            if value.starts_with('(') && value.ends_with(')') {
                let array_content = &value[1..value.len() - 1];
                let mut items = Vec::new();
                let mut current_item = String::new();
                let mut in_quotes = false;
                for ch in array_content.chars() {
                    match ch {
                        '"' => in_quotes = !in_quotes,
                        ' ' if !in_quotes => {
                            if !current_item.is_empty() {
                                items.push(current_item.clone());
                                current_item.clear();
                            }
                        }
                        _ => current_item.push(ch),
                    }
                }
                if !current_item.is_empty() {
                    items.push(current_item);
                }
                set_var(&format!("{}_LENGTH", key), &items.len().to_string());
                for (i, item) in items.iter().enumerate() {
                    set_var(&format!("{}_{}", key, i), item);
                }
                set_var(key, &items.join(" "));
            } else {
                set_var(key, value);
            }
        }
    }
}
pub fn load_config_file(path: &str) {
    let expanded_path = expand_vars(path);
    if let Ok(content) = std::fs::read_to_string(&expanded_path) {
        parse_config_content(&content);
    }
}
pub fn save_config_file(path: &str, keys: &[&str]) {
    let expanded_path = expand_vars(path);
    let mut content = String::new();
    content.push_str("# RSB Configuration File\n");
    content.push_str(&format!(
        "# Generated on {}\n\n",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    ));
    for key in keys {
        if has_var(key) {
            let value = get_var(key);
            if value.contains(' ') {
                content.push_str(&format!("{}=\"{}\"\n", key, value));
            } else {
                content.push_str(&format!("{}={}\n", key, value));
            }
        }
    }
    if let Some(parent) = std::path::Path::new(&expanded_path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&expanded_path, &content);
}
pub fn export_vars(path: &str) {
    let expanded_path = expand_vars(path);
    let all_vars = CTX.lock().unwrap().get_all_vars();
    let mut content = String::new();
    for (key, value) in all_vars.iter() {
        content.push_str(&format!("export {}='{}'\n", key, value));
    }
    if let Some(parent) = std::path::Path::new(&expanded_path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&expanded_path, &content);
}

// --- Bootstrap Functions ---
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_config_content() {
        rsb_bootstrap(&[]);
        let content = r#"
# This is a comment
KEY=VALUE
KEY_WITH_SPACES="value with spaces"
ARRAY=(item1 item2 "item 3")
        "#;
        parse_config_content(content);
        assert_eq!(get_var("KEY"), "VALUE");
        assert_eq!(get_var("KEY_WITH_SPACES"), "value with spaces");
        assert_eq!(get_var("ARRAY"), "item1 item2 item 3");
        assert_eq!(get_var("ARRAY_LENGTH"), "3");
        assert_eq!(get_var("ARRAY_0"), "item1");
        assert_eq!(get_var("ARRAY_1"), "item2");
        assert_eq!(get_var("ARRAY_2"), "item 3");
    }
}
fn setup_xdg_paths() {
    set_var("XDG_HOME", &expand_vars("$HOME/.local"));
    set_var("XDG_LIB", &expand_vars("$XDG_HOME/lib"));
    set_var("XDG_ETC", &expand_vars("$XDG_HOME/etc"));
    set_var("XDG_BIN", &expand_vars("$XDG_HOME/bin"));
    set_var("XDG_DATA", &expand_vars("$XDG_HOME/data"));
    set_var("XDG_TMP", &expand_vars("$HOME/.cache/tmp"));
}
fn setup_rsb_paths() {
    set_var("RSB_LIB", &expand_vars("$XDG_LIB/rsb"));
    set_var("RSB_BIN", &expand_vars("$XDG_BIN/rsb"));
    set_var("RSB_ETC", &expand_vars("$XDG_ETC/rsb"));
    set_var("RSB_DATA", &expand_vars("$XDG_DATA/rsb"));
    set_var("ODX_LIB", &expand_vars("$XDG_LIB/odx"));
    set_var("ODX_BIN", &expand_vars("$XDG_BIN/odx"));
    set_var("RSB_EXPORT", &expand_vars("$RSB_ETC/export.env"));
}
fn setup_standard_modes() {
    if std::env::var("DEBUG").is_ok() {
        set_var("DEBUG_MODE", "true");
    }
    if std::env::var("DEV").is_ok() {
        set_var("DEV_MODE", "true");
    }
    if std::env::var("QUIET").is_ok() {
        set_var("QUIET_MODE", "true");
    }
    if std::env::var("TRACE").is_ok() {
        set_var("TRACE_MODE", "true");
    }
}
fn parse_rsb_colors() {
    if let Ok(rsb_colors) = std::env::var("RSB_COLORS") {
        let mut colors = COLORS.lock().unwrap();
        let mut glyphs = GLYPHS.lock().unwrap();
        for part in rsb_colors.split(',') {
            let part = part.trim();
            if let Some((level, config)) = part.split_once(':') {
                let config = config.trim_matches(|c| c == '[' || c == ']');
                if let Some((color, glyph)) = config.split_once(';') {
                    colors.insert(level.to_string(), color.to_string());
                    glyphs.insert(level.to_string(), glyph.to_string());
                } else if !config.is_empty() {
                    if config.starts_with("\x1b[") {
                         colors.insert(level.to_string(), config.to_string());
                    } else {
                         glyphs.insert(level.to_string(), config.to_string());
                    }
                }
            }
        }
    }
}
fn setup_script_awareness(args: &[String]) {
    if args.is_empty() {
        return;
    }
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
    set_var(
        "PWD",
        &std::env::current_dir()
            .unwrap()
            .to_string_lossy()
            .to_string(),
    );
}
pub fn rsb_bootstrap(args: &[String]) {
    for (key, value) in std::env::vars() {
        set_var(&key, &value);
    }
    parse_rsb_colors();
    setup_xdg_paths();
    setup_rsb_paths();
    setup_standard_modes();
    setup_script_awareness(args);
}
