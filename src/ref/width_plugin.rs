

use crate::{File, Command, Stdio, UnicodeWidthStr};



/// Validate width input
pub fn validate_width(width_str: &str) -> Result<(), String> {
  match width_str.parse::<usize>() {
    Ok(w) if w >= 10 && w <= 200 => Ok(()),
    Ok(w) => Err(format!("Width {} out of range (10-200)", w)),
    Err(_) => Err("Width must be a number".to_string()),
  }
}

/// Width diagnostics subcommand
pub fn handle_width_command() {
    // Helper to run command with /dev/tty as stdin when available
    fn run_with_tty(mut cmd: Command) -> Option<String> {
        if let Ok(tty) = File::open("/dev/tty") {
            let _ = cmd.stdin(Stdio::from(tty));
        }
        cmd.output().ok().and_then(|o| String::from_utf8(o.stdout).ok())
    }

    // Gather tput cols (tty)
    let tput_cols_tty = {
        let mut c = Command::new("tput");
        c.arg("cols");
        run_with_tty(c).and_then(|s| s.trim().parse::<usize>().ok())
    };

    // Gather stty size (rows cols) via tty
    let stty_cols_tty = {
        let mut c = Command::new("stty");
        c.arg("size");
        run_with_tty(c).and_then(|s| {
            let parts: Vec<&str> = s.split_whitespace().collect();
            if parts.len() == 2 { parts[1].parse::<usize>().ok() } else { None }
        })
    };

    let effective = get_terminal_width();
    
    println!("Width diagnostics:");
    println!("  effective (get_terminal_width): {}", effective);
    println!("  tput cols (tty): {}", tput_cols_tty.map(|v| v.to_string()).unwrap_or_else(|| "N/A".to_string()));
    println!("  stty size cols (tty): {}", stty_cols_tty.map(|v| v.to_string()).unwrap_or_else(|| "N/A".to_string()));
}

/// Get terminal width with fallback to 80 columns
pub fn get_terminal_width() -> usize {
    // Helper to run with /dev/tty
    fn run_with_tty(mut cmd: Command) -> Option<String> {
        if let Ok(tty) = File::open("/dev/tty") {
            let _ = cmd.stdin(Stdio::from(tty));
        }
        cmd.output().ok().and_then(|o| String::from_utf8(o.stdout).ok())
    }

    // Try tput cols with tty (preferred)
    {
        let mut c = Command::new("tput");
        c.arg("cols");
        if let Some(out) = run_with_tty(c) {
        if let Ok(width) = out.trim().parse::<usize>() {
            if width >= 10 { return width; }
        }
        }
    }

    // Try stty size with tty
    {
        let mut c = Command::new("stty");
        c.arg("size");
        if let Some(out) = run_with_tty(c) {
        let parts: Vec<&str> = out.split_whitespace().collect();
        if parts.len() == 2 {
            if let Ok(width) = parts[1].trim().parse::<usize>() {
                if width >= 10 { return width; }
            }
        }
        }
    }

    80
}

pub fn get_display_width(text: &str) -> usize {
    let clean = strip_ansi_escapes::strip(text);
    let clean_str = String::from_utf8_lossy(&clean);
    UnicodeWidthStr::width(clean_str.as_ref())
}
