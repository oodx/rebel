use crate::context::{get_var, set_var};
use regex::Regex;
use std::io::{self, Write};

// --- Array Utilities ---
pub fn get_array(key: &str) -> Vec<String> {
    let val = get_var(key);
    val.split_whitespace().map(|s| s.to_string()).collect()
}
pub fn set_array(key: &str, arr: &[String]) {
    set_var(key, &arr.join(" "));
}
pub fn array_push(key: &str, val: &str) {
    let mut arr = get_array(key);
    arr.push(val.to_string());
    set_var(key, &arr.join(" "));
}

// --- User Interaction ---
pub fn prompt_user(msg: &str, default: Option<&str>) -> String {
    let mut input = String::new();
    if let Some(d) = default {
        print!("{} [{}]: ", msg, d);
    } else {
        print!("{}: ", msg);
    }
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).unwrap();
    let trimmed = input.trim();
    if trimmed.is_empty() && default.is_some() {
        default.unwrap().to_string()
    } else {
        trimmed.to_string()
    }
}
pub fn confirm_action(msg: &str, default: Option<bool>) -> bool {
    let mut input = String::new();
    let prompt = match default {
        Some(true) => " [Y/n]",
        Some(false) => " [y/N]",
        None => " [y/n]",
    };
    print!("{}{}: ", msg, prompt);
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).unwrap();
    match input.trim().to_lowercase().as_str() {
        "y" | "yes" => true,
        "n" | "no" => false,
        "" => default.unwrap_or(false),
        _ => false,
    }
}

// --- Output Formatting ---
pub fn glyph_stderr(level: &str, msg: &str) {
    let colors = crate::context::COLORS.lock().unwrap();
    let glyphs = crate::context::GLYPHS.lock().unwrap();
    let use_color = get_var("RSB_COLORS") != "false";

    let (color_key, glyph_key) = match level {
        "info" => ("cyan", "info"),
        "okay" => ("green", "okay"),
        "warn" => ("yellow", "warn"),
        "error" => ("red", "error"),
        "fatal" => ("red", "fatal"),
        "debug" => ("grey", "debug"),
        "trace" => ("grey", "trace"),
        _ => ("reset", " "),
    };

    let color = colors.get(color_key).unwrap();
    let space = " ".to_string();
    let glyph = glyphs.get(glyph_key).unwrap_or(&space);
    let reset = colors.get("reset").unwrap();
    let bold = colors.get("bold").unwrap();

    if use_color {
        eprintln!("{bold}{color}{glyph}{reset} {bold}{msg}{reset}");
    } else {
        eprintln!("[{}] {}", level.to_uppercase(), msg);
    }
}

// --- String Comparison ---
pub fn str_equals(a: &str, b: &str) -> bool { a == b }
pub fn str_matches(s: &str, re: &str) -> bool {
    Regex::new(re).map_or(false, |r| r.is_match(s))
}

// --- Numeric Comparison ---
pub fn num_eq(a: &str, b: &str) -> bool {
    a.parse::<f64>().unwrap_or_default() == b.parse::<f64>().unwrap_or_default()
}
pub fn num_lt(a: &str, b: &str) -> bool {
    a.parse::<f64>().unwrap_or_default() < b.parse::<f64>().unwrap_or_default()
}
pub fn num_gt(a: &str, b: &str) -> bool {
    a.parse::<f64>().unwrap_or_default() > b.parse::<f64>().unwrap_or_default()
}

// --- String Manipulation (for param! macro) ---
pub fn str_sub(s: &str, start: usize, len: Option<usize>) -> String {
    let end = len.map(|l| start.saturating_add(l)).unwrap_or(s.len());
    s.chars().skip(start).take(end.saturating_sub(start)).collect()
}
pub fn str_prefix(s: &str, pattern: &str, longest: bool) -> String {
    // This is a simplified implementation. A real implementation would handle glob patterns.
    if longest {
        s.strip_prefix(pattern).unwrap_or(s).to_string()
    } else {
        s.strip_prefix(pattern).unwrap_or(s).to_string()
    }
}
pub fn str_suffix(s: &str, pattern: &str, longest: bool) -> String {
    // This is a simplified implementation. A real implementation would handle glob patterns.
    if longest {
        s.strip_suffix(pattern).unwrap_or(s).to_string()
    } else {
        s.strip_suffix(pattern).unwrap_or(s).to_string()
    }
}
pub fn str_replace(s: &str, from: &str, to: &str, all: bool) -> String {
    if all { s.replace(from, to) } else { s.replacen(from, to, 1) }
}
pub fn str_upper(s: &str, first: bool) -> String {
    if !first {
        s.to_uppercase()
    } else {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }
}
pub fn str_lower(s: &str, first: bool) -> String {
    if !first {
        s.to_lowercase()
    } else {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
        }
    }
}
