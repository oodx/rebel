use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub function: String,
    pub args: Vec<String>,
    pub timestamp: std::time::SystemTime,
    pub context_snapshot: HashMap<String, String>,
}

pub struct Context {
    vars: HashMap<String, String>,
}

impl Context {
    pub fn new() -> Self {
        Context { vars: HashMap::new() }
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
        let mut result = text.to_string();
        let braced_re = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap();
        result = braced_re.replace_all(&result, |caps: &regex::Captures| {
            self.vars.get(&caps[1]).cloned().unwrap_or_default()
        }).to_string();
        let simple_re = Regex::new(r"\$([A-Za-z_][A-Za-z0-9_]*)").unwrap();
        result = simple_re.replace_all(&result, |caps: &regex::Captures| {
            self.vars.get(&caps[1]).cloned().unwrap_or_default()
        }).to_string();
        result
    }
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
    pub(crate) static ref FUNCTION_REGISTRY: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    pub(crate) static ref CALL_STACK: Arc<Mutex<Vec<CallFrame>>> = Arc::new(Mutex::new(Vec::new()));
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

pub fn set_var<K: Into<String>, V: Into<String>>(key: K, value: V) { CTX.lock().unwrap().set(key, value); }
pub fn get_var(key: &str) -> String { CTX.lock().unwrap().get(key) }
pub fn has_var(key: &str) -> bool { CTX.lock().unwrap().has(key) }
pub fn unset_var(key: &str) { CTX.lock().unwrap().vars.remove(key); }
pub fn expand_vars(text: &str) -> String { CTX.lock().unwrap().expand(text) }
pub fn register_function(name: &str, description: &str) { FUNCTION_REGISTRY.lock().unwrap().insert(name.to_string(), description.to_string()); }
pub fn list_functions() -> Vec<(String, String)> {
    let mut funcs: Vec<_> = FUNCTION_REGISTRY.lock().unwrap().iter().map(|(k, v)| (k.clone(), v.clone())).collect();
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
pub fn pop_call() -> Option<CallFrame> { CALL_STACK.lock().unwrap().pop() }
pub fn get_call_stack() -> Vec<CallFrame> { CALL_STACK.lock().unwrap().clone() }
pub fn show_help() { /* ... */ }
pub fn show_functions() { /* ... */ }
pub fn show_call_stack() { /* ... */ }
pub fn parse_config_content(_content: &str) { /* ... */ }
pub fn load_config_file(_path: &str) { /* ... */ }
pub fn save_config_file(_path: &str, _keys: &[&str]) { /* ... */ }
pub fn export_vars(_path: &str) { /* ... */ }
fn setup_xdg_paths() { /* ... */ }
fn setup_rsb_paths() { /* ... */ }
fn setup_standard_modes() { /* ... */ }
fn parse_rsb_colors() { /* ... */ }
fn setup_script_awareness(_args: &[String]) { /* ... */ }
pub fn rsb_bootstrap(_args: &[String]) { /* ... */ }
