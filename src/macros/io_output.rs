// --- I/O Macros ---
// Namespaced re-exports for selective imports
pub use crate::{readline, stderr, colored, echo, printf, info, okay, warn, error, fatal, debug, trace};
#[macro_export]
macro_rules! readline {
    () => {
        {
            let mut input = String::new();
            match std::io::stdin().read_line(&mut input) {
                Ok(_) => input.trim().to_string(),
                Err(_) => String::new(),
            }
        }
    };
    ($prompt:expr) => {
        {
            eprint!("{}", $prompt);
            let _ = std::io::stderr().flush();
            let mut input = String::new();
            match std::io::stdin().read_line(&mut input) {
                Ok(_) => input.trim().to_string(),
                Err(_) => String::new(),
            }
        }
    };
}

#[macro_export]
macro_rules! stderr {
    ($($arg:tt)*) => {
        {
            let msg = format!($($arg)*);
            eprintln!("{}", msg);
        }
    };
}

// Returns a string with color placeholders expanded, without printing.
#[macro_export]
macro_rules! colored {
    ($($arg:tt)*) => {
        {
            let s = format!($($arg)*);
            $crate::utils::expand_colors(&s)
        }
    };
}

// --- Output Macros ---
#[macro_export]
macro_rules! echo { ($($arg:tt)*) => { println!("{}", $crate::context::expand_vars(&format!($($arg)*))); }; }
#[macro_export]
macro_rules! printf { ($($arg:tt)*) => { print!("{}", $crate::context::expand_vars(&format!($($arg)*))); }; }
#[macro_export]
macro_rules! info { ($($arg:tt)*) => { $crate::utils::glyph_stderr("info", &format!($($arg)*)); }; }
#[macro_export]
macro_rules! okay { ($($arg:tt)*) => { $crate::utils::glyph_stderr("okay", &format!($($arg)*)); }; }
#[macro_export]
macro_rules! warn { ($($arg:tt)*) => { $crate::utils::glyph_stderr("warn", &format!($($arg)*)); }; }
#[macro_export]
macro_rules! error { ($($arg:tt)*) => { $crate::utils::glyph_stderr("error", &format!($($arg)*)); }; }
#[macro_export]
macro_rules! fatal { ($($arg:tt)*) => { $crate::utils::glyph_stderr("fatal", &format!($($arg)*)); }; }
#[macro_export]
macro_rules! debug { ($($arg:tt)*) => { $crate::utils::glyph_stderr("debug", &format!($($arg)*)); }; }
#[macro_export]
macro_rules! trace { ($($arg:tt)*) => { $crate::utils::glyph_stderr("trace", &format!($($arg)*)); }; }
