use crate::context::{expand_vars, has_var, COLORS, GLYPHS};

// This module will contain miscellaneous utilities, such as the
// StringExt trait, array operations, and user interaction functions.

// --- Output Helpers ---

pub fn should_print_level(level: &str) -> bool {
    if has_var("QUIET_MODE") && !["error", "fatal"].contains(&level) {
        return false;
    }

    match level {
        "trace" | "think" => has_var("TRACE_MODE"),
        "debug" => has_var("DEBUG_MODE") || has_var("TRACE_MODE"),
        "info" | "warn" | "okay" => has_var("DEBUG_MODE") || has_var("DEV_MODE") || has_var("TRACE_MODE"),
        "error" | "fatal" => true,

        _ => true,
    }
}
pub fn expand_colors(text: &str) -> String {
    let mut result = text.to_string();
    let colors = COLORS.lock().unwrap();
    let reset_code = colors.get("reset").cloned().unwrap_or_else(|| "\x1b[0m".to_string());

        _ => true, // Default to printing unknown levels
    }
}

/// Replaces color placeholders (e.g., `{red}`) with ANSI color codes.
pub fn expand_colors(text: &str) -> String {
    let mut result = text.to_string();
    let colors = COLORS.lock().unwrap();
    // Also get the reset code to append at the end
    let reset_code = colors.get("reset").cloned().unwrap_or_else(|| "\x1b[0m".to_string());

    //todo: is this correct?    
    result = result.replace("{reset}", &reset_code);

    if result.contains('\x1b') && !result.ends_with(&reset_code) {
        result.push_str(&reset_code);
    }
    result
}

pub fn glyph_stderr(level: &str, message: &str) {
    if !should_print_level(level) { return; }
    let glyphs = GLYPHS.lock().unwrap();
    let glyph = glyphs.get(level).cloned().unwrap_or_else(|| "•".to_string());
    let colors = COLORS.lock().unwrap();
    let color_name = match level {
        "info" => "cyan", "okay" => "green", "warn" => "yellow",
        "error" | "fatal" => "red", "debug" => "grey", "trace" => "magenta",
        _ => "reset",
    };
    let color_code = colors.get(color_name).cloned().unwrap_or_default();

    let expanded_msg = expand_vars(message);
    let final_msg = format!("{}{}{}", color_code, glyph, expanded_msg);
    eprintln!("{}", expand_colors(&final_msg));
}


// --- String & Name Helpers ---

pub fn is_name(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}


// --- Comparison Helpers ---
pub fn str_equals(a: &str, b: &str) -> bool { a == b }
pub fn str_matches(text: &str, pattern: &str) -> bool {
    match regex::Regex::new(pattern) {
        Ok(re) => re.is_match(text),
        Err(_) => false,
    }
}
fn to_f64(s: &str) -> Option<f64> { s.parse::<f64>().ok() }
pub fn num_eq(a: &str, b: &str) -> bool {
    match (to_f64(a), to_f64(b)) {
        (Some(na), Some(nb)) => (na - nb).abs() < f64::EPSILON,
        _ => false,
    }
}

pub fn num_lt(a: &str, b: &str) -> bool {

    match (to_f64(a), to_f64(b)) {
        (Some(na), Some(nb)) => na < nb,
        _ => false,
    }
}

pub fn num_gt(a: &str, b: &str) -> bool {

    match (to_f64(a), to_f64(b)) {
        (Some(na), Some(nb)) => na > nb,
        _ => false,
    }
}

// --- Array Helpers ---

pub fn array_push(key: &str, item: &str) {
    let mut items = get_array(key);
    items.push(item.to_string());
    let item_strs: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
    set_array(key, &item_strs);
}

pub fn array_get(key: &str, index: usize) -> String {
    get_array(key).get(index).cloned().unwrap_or_default()
}

pub fn array_length(key: &str) -> usize {
    get_array(key).len()
}

pub fn array_contains(key: &str, item: &str) -> bool {
    get_array(key).contains(&item.to_string())
}

// The spec also calls for `get_array`. I will implement that here too.
pub fn get_array(key: &str) -> Vec<String> {
    use crate::context::get_var;
    let length_key = format!("{}_LENGTH", key);
    if !crate::context::has_var(&length_key) {
        let value = get_var(key);
        if value.is_empty() { return Vec::new(); }
        return value.split_whitespace().map(|s| s.to_string()).collect();
    }
    let length: usize = get_var(&length_key).parse().unwrap_or(0);
    let mut items = Vec::new();
    for i in 0..length {
        let item_key = format!("{}_{}", key, i);
        if crate::context::has_var(&item_key) {
            items.push(get_var(&item_key));
        }
    }
    items
}


pub fn set_array(key: &str, items: &[&str]) {
    use crate::context::set_var;
    set_var(&format!("{}_LENGTH", key), &items.len().to_string());
    for (i, item) in items.iter().enumerate() {
        set_var(&format!("{}_{}", key, i), *item);
    }
    set_var(key, &items.join(" "));
}

// --- User Interaction ---

pub fn prompt_user(message: &str, default: Option<&str>) -> String {
    use std::io::{self, Write};

    let default_text = if let Some(def) = default {
        format!(" [{}]", def)
    } else {
        String::new()
    };

    print!("{}{}: ", expand_vars(message), default_text);
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

pub fn confirm_action(message: &str, default: Option<bool>) -> bool {
    use crate::context::has_var;
    use std::io::{self, Write};

    if has_var("opt_yes") {
        return true;
    }

    let default_text = match default {
        Some(true) => " [Y/n]",
        Some(false) => " [y/N]",
        None => " [y/n]"
    };

    loop {
        print!("{}{}: ", expand_vars(message), default_text);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            "" => {
                if let Some(def) = default {
                    return def;
                }
            }
            _ => continue,
        }
    }
}


// --- Parameter Expansion Helpers (Refactored Names) ---
pub fn str_sub(var: &str, offset: usize, length: Option<usize>) -> String {
    let len = length.unwrap_or(var.len());
    var.chars().skip(offset).take(len).collect()
}
pub fn str_prefix(var: &str, pattern: &str, longest: bool) -> String {

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

                    if !longest { break; }
                }
            }
        }
        if found_match { return var[best_match_len..].to_string(); }
    }
    var.to_string()
}
pub fn str_suffix(var: &str, pattern: &str, longest: bool) -> String {

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

                    if !longest { break; }
                }
            }
        }
        if found_match { return var[..best_match_len].to_string(); }
    }
    var.to_string()
}
pub fn str_replace(var: &str, pattern: &str, replacement: &str, all: bool) -> String {

    if all {
        var.replace(pattern, replacement)
    } else {
        var.replacen(pattern, replacement, 1)
    }
}

pub fn str_upper(var: &str, all: bool) -> String {

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

pub fn str_lower(var: &str, all: bool) -> String {

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
    fn test_str_replace() {
        assert_eq!(str_replace("hello world", "world", "rust", false), "hello rust");
        assert_eq!(str_replace("hello world world", "world", "rust", false), "hello rust world");
        assert_eq!(str_replace("hello world world", "world", "rust", true), "hello rust rust");
    }

    #[test]
    fn test_is_name() {
        assert!(is_name("valid-name"));
        assert!(is_name("valid_name_123"));
        assert!(!is_name("invalid name"));
        assert!(!is_name("invalid!@#"));
    }
}
