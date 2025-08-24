use crate::context::{expand_vars, has_var, COLORS, GLYPHS};

// This module will contain miscellaneous utilities, such as the
// StringExt trait, array operations, and user interaction functions.

// --- Output Helpers ---

/// Determines if a message of a given level should be printed based on context variables.
pub fn should_print_level(level: &str) -> bool {
    if has_var("QUIET_MODE") && !["error", "fatal"].contains(&level) {
        return false;
    }

    match level {
        "trace" | "think" => has_var("TRACE_MODE"),
        "debug" => has_var("DEBUG_MODE") || has_var("TRACE_MODE"),
        "info" | "warn" | "okay" => has_var("DEBUG_MODE") || has_var("DEV_MODE") || has_var("TRACE_MODE"),
        "error" | "fatal" => true,
        _ => true, // Default to printing unknown levels
    }
}

/// Replaces color placeholders (e.g., `{red}`) with ANSI color codes.
pub fn expand_colors(text: &str) -> String {
    let mut result = text.to_string();
    let colors = COLORS.lock().unwrap();
    // Also get the reset code to append at the end
    let reset_code = colors.get("reset").cloned().unwrap_or_else(|| "\x1b[0m".to_string());

    for (name, code) in colors.iter() {
        if name != "reset" {
            result = result.replace(&format!("{{{}}}", name), code);
        }
    }

    // Replace the specific {reset} placeholder
    result = result.replace("{reset}", &reset_code);

    // Ensure reset is applied at the end if a color was used
    if result.contains('\x1b') && !result.ends_with(&reset_code) {
        result.push_str(&reset_code);
    }
    result
}

/// Prints a formatted, colored, and glyph-prefixed message to stderr.
pub fn glyph_stderr(level: &str, message: &str) {
    if !should_print_level(level) {
        return;
    }

    let glyphs = GLYPHS.lock().unwrap();
    let glyph = glyphs.get(level).cloned().unwrap_or_else(|| "•".to_string());

    let colors = COLORS.lock().unwrap();
    let color_name = match level {
        "info" => "cyan",
        "okay" => "green",
        "warn" => "yellow",
        "error" | "fatal" => "red",
        "debug" => "grey",
        "trace" => "magenta",
        _ => "reset",
    };
    let color_code = colors.get(color_name).cloned().unwrap_or_default();

    // Manually format to avoid issues with user-provided format strings
    let expanded_msg = expand_vars(message);
    let final_msg = format!("{}{}{}", color_code, glyph, expanded_msg);
    eprintln!("{}", expand_colors(&final_msg));
}


/// Checks if a string is a valid name (alphanumeric + '-' and '_').
pub fn is_name(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}


// --- Comparison Helpers ---

/// Checks if two strings are equal.
pub fn str_equals(a: &str, b: &str) -> bool {
    a == b
}

/// Checks if a string matches a regex pattern.
pub fn str_matches(text: &str, pattern: &str) -> bool {
    match regex::Regex::new(pattern) {
        Ok(re) => re.is_match(text),
        Err(_) => false, // Invalid regex pattern
    }
}

/// Parses a string into an f64, returning an option.
fn to_f64(s: &str) -> Option<f64> {
    s.parse::<f64>().ok()
}

/// Checks if two strings are numerically equal.
pub fn num_equals(a: &str, b: &str) -> bool {
    match (to_f64(a), to_f64(b)) {
        (Some(na), Some(nb)) => (na - nb).abs() < f64::EPSILON,
        _ => false,
    }
}

/// Checks if the first string is numerically less than the second.
pub fn num_less_than(a: &str, b: &str) -> bool {
    match (to_f64(a), to_f64(b)) {
        (Some(na), Some(nb)) => na < nb,
        _ => false,
    }
}

/// Checks if the first string is numerically greater than the second.
pub fn num_greater_than(a: &str, b: &str) -> bool {
    match (to_f64(a), to_f64(b)) {
        (Some(na), Some(nb)) => na > nb,
        _ => false,
    }
}



// --- Array Helpers ---

/// Sets an array variable in the context.
/// This stores the array as both a space-separated string
/// and as indexed variables (e.g., MY_ARRAY_0, MY_ARRAY_1).
pub fn set_array(key: &str, items: &[&str]) {
    use crate::context::set_var;
    set_var(&format!("{}_LENGTH", key), &items.len().to_string());
    for (i, item) in items.iter().enumerate() {
        set_var(&format!("{}_{}", key, i), *item);
    }
    set_var(key, &items.join(" "));
}



// --- Parameter Expansion Helpers ---

/// Implements `${var:-default}`
pub fn var_default(var: &str, default: &str) -> String {
    if var.is_empty() {
        default.to_string()
    } else {
        var.to_string()
    }
}

/// Implements `${var:+alternate}`
pub fn var_alternate(var: &str, alternate: &str) -> String {
    if var.is_empty() {
        String::new()
    } else {
        alternate.to_string()
    }
}

/// Implements `${var:offset:length}`
pub fn var_substring(var: &str, offset: usize, length: Option<usize>) -> String {
    let len = length.unwrap_or(var.len());
    var.chars().skip(offset).take(len).collect()
}

/// Implements `${var#pattern}` and `${var##pattern}`
pub fn var_trim_prefix(var: &str, pattern: &str, longest: bool) -> String {

    if let Ok(p) = glob::Pattern::new(pattern) {
        let mut best_match_len = 0;
        let mut found_match = false;
        for i in 0..=var.len() {
            let sub = &var[i..];
            if p.matches(sub) {
                let current_match_len = var.len() - sub.len();
                if !found_match || (longest && current_match_len > best_match_len) || (!longest && current_match_len < best_match_len) {
                    best_match_len = current_match_len;
                    found_match = true;
                    if !longest { break; } // For shortest match, take the first one
                }
            }
        }
        if found_match {
            return var[best_match_len..].to_string();
        }
    }
    var.to_string()

}

/// Implements `${var%pattern}` and `${var%%pattern}`
pub fn var_trim_suffix(var: &str, pattern: &str, longest: bool) -> String {

    if let Ok(p) = glob::Pattern::new(pattern) {
        let mut best_match_len = 0;
        let mut found_match = false;
        for i in (0..=var.len()).rev() {
            let sub = &var[..i];
            if p.matches(sub) {
                let current_match_len = i;
                 if !found_match || (longest && current_match_len < best_match_len) || (!longest && current_match_len > best_match_len) {
                    best_match_len = current_match_len;
                    found_match = true;
                    if !longest { break; } // For shortest match, take the first one
                }
            }
        }
        if found_match {
            return var[..best_match_len].to_string();
        }
    }
    var.to_string()

}

/// Implements `${var/pattern/string}` and `${var//pattern/string}`
pub fn var_replace(var: &str, pattern: &str, replacement: &str, all: bool) -> String {
    if all {
        var.replace(pattern, replacement)
    } else {
        var.replacen(pattern, replacement, 1)
    }
}

/// Implements `${var^}` and `${var^^}`
pub fn var_case_upper(var: &str, all: bool) -> String {
    if all {
        var.to_uppercase()
    } else {
        let mut c = var.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }
}

/// Implements `${var,}` and `${var,,}`
pub fn var_case_lower(var: &str, all: bool) -> String {
    if all {
        var.to_lowercase()
    } else {
        let mut c = var.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_var_default() {
        assert_eq!(var_default("", "default"), "default");
        assert_eq!(var_default("value", "default"), "value");
    }

    #[test]
    fn test_var_replace() {
        assert_eq!(var_replace("hello world", "world", "rust", false), "hello rust");
        assert_eq!(var_replace("hello world world", "world", "rust", false), "hello rust world");
        assert_eq!(var_replace("hello world world", "world", "rust", true), "hello rust rust");
    }

    #[test]
    fn test_is_name() {
        assert!(is_name("valid-name"));
        assert!(is_name("valid_name_123"));
        assert!(!is_name("invalid name"));
        assert!(!is_name("invalid!@#"));
    }
}
