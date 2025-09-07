// --- Conversion / Text Macros ---
// Namespaced re-exports for selective imports
pub use crate::{to_number, param, str_in, str_explode, str_trim, str_len};
#[macro_export]
macro_rules! to_number {
    ($text:expr) => {
        {
            $text.trim().parse::<i32>().unwrap_or(0)
        }
    };
    ($text:expr, default: $default:expr) => {
        {
            $text.trim().parse::<i32>().unwrap_or($default)
        }
    };
}

// --- Parameter Expansion Macro ---
#[macro_export]
macro_rules! param {
    ($var:expr) => { $crate::context::get_var($var) };
    ($var:expr, default: $default:expr) => {{
        let val = $crate::context::get_var($var);
        if val.is_empty() { $default.to_string() } else { val }
    }};
    ($var:expr, alt: $alt:expr) => {{
        let val = $crate::context::get_var($var);
        if val.is_empty() { String::new() } else { $alt.to_string() }
    }};

    ($var:expr, sub: $start:expr) => { $crate::utils::str_sub(&$crate::context::get_var($var), $start, None) };
    ($var:expr, sub: $start:expr, $len:expr) => { $crate::utils::str_sub(&$crate::context::get_var($var), $start, Some($len)) };
    ($var:expr, prefix: $pattern:expr) => { $crate::utils::str_prefix(&$crate::context::get_var($var), $pattern, false) };
    ($var:expr, prefix: $pattern:expr, longest) => { $crate::utils::str_prefix(&$crate::context::get_var($var), $pattern, true) };
    ($var:expr, suffix: $pattern:expr) => { $crate::utils::str_suffix(&$crate::context::get_var($var), $pattern, false) };
    ($var:expr, suffix: $pattern:expr, longest) => { $crate::utils::str_suffix(&$crate::context::get_var($var), $pattern, true) };
    ($var:expr, replace: $from:expr => $to:expr) => { $crate::utils::str_replace(&$crate::context::get_var($var), $from, $to, false) };
    ($var:expr, replace: $from:expr => $to:expr, all) => { $crate::utils::str_replace(&$crate::context::get_var($var), $from, $to, true) };
    ($var:expr, upper) => { $crate::utils::str_upper(&$crate::context::get_var($var), true) };
    ($var:expr, lower) => { $crate::utils::str_lower(&$crate::context::get_var($var), true) };
    ($var:expr, upper: first) => { $crate::utils::str_upper(&$crate::context::get_var($var), false) };
    ($var:expr, lower: first) => { $crate::utils::str_lower(&$crate::context::get_var($var), false) };

    ($var:expr, len) => { $crate::context::get_var($var).len() };
}

// --- String Utilities ---
#[macro_export]
macro_rules! str_in {
    ($needle:expr, in: $haystack:expr) => {
        $haystack.contains($needle)
    };
}

#[macro_export]
macro_rules! str_explode {
    ($string:expr, on: $delim:expr, into: $arr_name:expr) => {
        {
            let items: Vec<&str> = $string.split($delim).collect();
            $crate::utils::set_array($arr_name, &items);
        }
    };
}

#[macro_export]
macro_rules! str_trim {
    ($var:expr) => {
        $crate::context::get_var($var).trim().to_string()
    };
}

#[macro_export]
macro_rules! str_len {
    ($var:expr) => {
        $crate::context::get_var($var).len()
    };
}
