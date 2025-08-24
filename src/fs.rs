use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use regex::Regex;

// --- File System Checks (for test! macro) ---
pub fn is_entity(path: &str) -> bool { Path::new(path).exists() }
pub fn is_file(path: &str) -> bool { Path::new(path).is_file() }
pub fn is_dir(path: &str) -> bool { Path::new(path).is_dir() }
pub fn is_link(path: &str) -> bool { Path::new(path).is_symlink() }
pub fn is_readable(path: &str) -> bool { fs::metadata(path).map(|m| !m.permissions().readonly()).unwrap_or(false) }
pub fn is_writable(path: &str) -> bool { fs::metadata(path).map(|m| m.permissions().readonly()).unwrap_or(false) }
pub fn is_executable(path: &str) -> bool {
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    #[cfg(unix)]
    return fs::metadata(path).map(|m| m.permissions().mode() & 0o111 != 0).unwrap_or(false);
    #[cfg(not(unix))]
    return true; // Simplified for non-unix
}
pub fn is_nonempty_file(path: &str) -> bool {
    fs::metadata(path).map(|m| m.len() > 0).unwrap_or(false)
}

// --- File I/O ---
pub fn read_file(path: &str) -> String {
    let mut content = String::new();
    if let Ok(mut file) = File::open(path) {
        file.read_to_string(&mut content).unwrap_or_default();
    }
    content
}
pub fn write_file(path: &str, content: &str) -> bool {
    File::create(path).and_then(|mut f| f.write_all(content.as_bytes())).is_ok()
}
pub fn file_append(path: &str, content: &str) -> bool {
    OpenOptions::new().create(true).append(true).open(path)
        .and_then(|mut f| f.write_all(content.as_bytes())).is_ok()
}
pub fn file_out(path: &str, content: &str) -> bool { write_file(path, content) }

// --- Path Manipulation ---
pub fn path_canon(path: &str) -> Result<String, std::io::Error> {
    let p = PathBuf::from(path);
    let abs_path = if p.is_absolute() { p } else { std::env::current_dir()?.join(p) };
    Ok(abs_path.to_string_lossy().to_string())
}
pub fn path_split(path: &str) -> HashMap<String, String> {
    let p = Path::new(path);
    let mut map = HashMap::new();
    map.insert("path".to_string(), path.to_string());
    map.insert("parent".to_string(), p.parent().unwrap_or(p).to_string_lossy().to_string());
    map.insert("filename".to_string(), p.file_name().unwrap_or_default().to_string_lossy().to_string());
    map.insert("stem".to_string(), p.file_stem().unwrap_or_default().to_string_lossy().to_string());
    map.insert("extension".to_string(), p.extension().unwrap_or_default().to_string_lossy().to_string());
    map
}

// --- Other File Utilities ---
pub fn chmod(path: &str, mode_str: &str) -> Result<(), std::io::Error> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = u32::from_str_radix(mode_str, 8).unwrap_or(0o644);
        let perms = fs::Permissions::from_mode(mode);
        fs::set_permissions(path, perms)?;
    }
    #[cfg(not(unix))]
    {
        // chmod is a no-op on non-unix systems in this simplified implementation
        let _ = (path, mode_str);
    }
    Ok(())
}
pub fn backup_file(path: &str, suffix: &str) -> Result<String, std::io::Error> {
    let backup_path = format!("{}{}", path, suffix);
    fs::copy(path, &backup_path)?;
    Ok(backup_path)
}
pub fn extract_meta_from_file(path: &str) -> HashMap<String, String> {
    let content = read_file(path);
    let mut meta = HashMap::new();
    let re = Regex::new(r"#\s*@(\w+):\s*(.*)").unwrap();
    for line in content.lines() {
        if let Some(caps) = re.captures(line) {
            meta.insert(caps[1].to_string(), caps[2].trim().to_string());
        }
    }
    meta
}
