# RSB Standard Library for Bash Script Porting

## What We Already Have (Strong Foundation)

### ✅ Stream Processing & Pipes
```rust
cat!("file.txt")
    .grep("pattern")
    .cut(2, ",")
    .head(10)
    .to_file("output.txt");
```

### ✅ Variable Expansion
```rust
let path = var!("$HOME/.config/$APP_NAME/settings.conf");
echo!("Processing {blue}$PROJECT{reset} v{yellow}$VERSION{reset}");
```

### ✅ Basic File Operations
```rust
let content = read_file("data.txt");
write_file("output.txt", &processed_data);
```

### ✅ Argument Handling
```rust
let target = args.get_or(1, "debug");
let clean = args.has_pop("--clean");
let features = args.get_array("features");
```

## Critical Gaps for Bash Porting

### 1. Parameter Expansion Patterns

**Bash patterns we need RSB equivalents for:**

```bash
# Bash parameter expansion
${var:-default}        # Use default if var is unset/empty
${var:+alternate}      # Use alternate if var is set  
${var:offset:length}   # Substring extraction
${var#pattern}         # Remove shortest match from beginning
${var##pattern}        # Remove longest match from beginning
${var%pattern}         # Remove shortest match from end
${var%%pattern}        # Remove longest match from end
${var/pattern/string}  # Replace first match
${var//pattern/string} # Replace all matches
${var^}                # Uppercase first char
${var^^}               # Uppercase all
${var,}                # Lowercase first char  
${var,,}               # Lowercase all
${#var}                # String length
```

**Proposed RSB API:**
```rust
// Parameter expansion functions
pub fn var_default(var: &str, default: &str) -> String;        // ${var:-default}
pub fn var_alternate(var: &str, alternate: &str) -> String;    // ${var:+alternate}  
pub fn var_substring(var: &str, offset: usize, length: Option<usize>) -> String;
pub fn var_trim_prefix(var: &str, pattern: &str, longest: bool) -> String;
pub fn var_trim_suffix(var: &str, pattern: &str, longest: bool) -> String;
pub fn var_replace(var: &str, pattern: &str, replacement: &str, all: bool) -> String;
pub fn var_case_upper(var: &str, all: bool) -> String;
pub fn var_case_lower(var: &str, all: bool) -> String;

// Macro interface for bash-like syntax
macro_rules! param {
    ($var:expr, default: $default:expr) => { var_default(&get_var($var), $default) };
    ($var:expr, alt: $alt:expr) => { var_alternate(&get_var($var), $alt) };
    ($var:expr, sub: $offset:expr, $length:expr) => { var_substring(&get_var($var), $offset, Some($length)) };
    ($var:expr, prefix: $pattern:expr) => { var_trim_prefix(&get_var($var), $pattern, false) };
    ($var:expr, suffix: $pattern:expr) => { var_trim_suffix(&get_var($var), $pattern, false) };
    ($var:expr, replace: $from:expr => $to:expr) => { var_replace(&get_var($var), $from, $to, false) };
}

// Usage examples
let config_file = param!("CONFIG_FILE", default: "app.conf");
let base_name = param!("FILENAME", suffix: ".txt");
let clean_path = param!("PATH", replace: "//" => "/");
```

### 2. Conditional Expressions & Tests

**Bash tests we need:**
```bash
[[ -f file ]]          # File exists and is regular file
[[ -d dir ]]           # Directory exists
[[ -r file ]]          # File is readable
[[ -w file ]]          # File is writable  
[[ -x file ]]          # File is executable
[[ -s file ]]          # File exists and is not empty
[[ -n string ]]        # String is not empty
[[ -z string ]]        # String is empty
[[ str1 == str2 ]]     # String equality
[[ str =~ regex ]]     # Regex match
[[ num1 -eq num2 ]]    # Numeric equality
[[ num1 -lt num2 ]]    # Numeric less than
```

**Proposed RSB API:**
```rust
// File test functions (extend existing is_file, is_dir)
pub fn is_readable(path: &str) -> bool;
pub fn is_writable(path: &str) -> bool;  
pub fn is_executable(path: &str) -> bool;
pub fn is_nonempty_file(path: &str) -> bool;

// String test functions (extend existing)
pub fn str_equals(a: &str, b: &str) -> bool;
pub fn str_matches(text: &str, pattern: &str) -> bool;  // Regex
pub fn str_contains(text: &str, substr: &str) -> bool;
pub fn str_starts_with(text: &str, prefix: &str) -> bool;
pub fn str_ends_with(text: &str, suffix: &str) -> bool;

// Numeric comparison functions
pub fn num_equals(a: &str, b: &str) -> bool;
pub fn num_less_than(a: &str, b: &str) -> bool;
pub fn num_greater_than(a: &str, b: &str) -> bool;

// Conditional macro for bash-like syntax
macro_rules! test {
    (-f $path:expr) => { is_file($path) };
    (-d $path:expr) => { is_dir($path) };
    (-r $path:expr) => { is_readable($path) };
    (-w $path:expr) => { is_writable($path) };
    (-x $path:expr) => { is_executable($path) };
    (-s $path:expr) => { is_nonempty_file($path) };
    (-n $str:expr) => { !is_empty($str) };
    (-z $str:expr) => { is_empty($str) };
    ($a:expr == $b:expr) => { str_equals($a, $b) };
    ($a:expr =~ $pattern:expr) => { str_matches($a, $pattern) };
    ($a:expr -eq $b:expr) => { num_equals($a, $b) };
    ($a:expr -lt $b:expr) => { num_less_than($a, $b) };
}

// Usage examples
if test!(-f "config.txt") && test!(-r "config.txt") {
    load_config!("config.txt");
}

if test!(get_var("VERSION") =~ r"^\d+\.\d+\.\d+$") {
    echo!("Valid version format");
}
```

### 3. Advanced String Operations

**Missing string functions for sed/awk-like operations:**

```rust
// String manipulation (extend StringExt trait)
impl StringExt for String {
    // Advanced sed-like operations
    fn sed_address(&self, line_num: usize, command: &str) -> String;  // 5s/old/new/
    fn sed_range(&self, start: usize, end: usize, command: &str) -> String;  // 1,5s/old/new/
    fn sed_delete(&self, pattern: &str) -> String;  // /pattern/d
    fn sed_print(&self, pattern: &str) -> Vec<String>;  // /pattern/p
    
    // Advanced awk-like operations  
    fn awk(&self, program: &str) -> String;  // Simple awk programs
    fn awk_field(&self, field: usize, separator: &str) -> Vec<String>;  // $1, $2, etc
    fn awk_nf(&self, separator: &str) -> Vec<usize>;  // Number of fields per line
    fn awk_nr(&self) -> usize;  // Number of records/lines
    
    // String processing
    fn trim_whitespace(&self) -> String;
    fn trim_chars(&self, chars: &str) -> String;
    fn pad_left(&self, width: usize, fill: char) -> String;
    fn pad_right(&self, width: usize, fill: char) -> String;
    fn split_lines(&self) -> Vec<String>;
    fn split_on(&self, delimiter: &str) -> Vec<String>;
    fn join_lines(&self, separator: &str) -> String;
    
    // Numeric operations on strings
    fn sum_numbers(&self) -> f64;  // Sum all numbers found in text
    fn count_matches(&self, pattern: &str) -> usize;
    fn extract_numbers(&self) -> Vec<f64>;
    
    // Advanced filtering
    fn filter_lines<F>(&self, predicate: F) -> String where F: Fn(&str) -> bool;
    fn transform_lines<F>(&self, transform: F) -> String where F: Fn(&str) -> String;
}

// Usage examples
let processed = content
    .sed_range(1, 10, "s/old/new/g")  // Replace in lines 1-10
    .awk_field(2, ",")                // Extract 2nd field
    .join("\n");

let numbers = data
    .extract_numbers()
    .iter()
    .map(|n| n.to_string())
    .collect::<Vec<_>>()
    .join(" ");
```

### 4. File System Operations

**Missing bash file operations:**
```rust
// File system helpers
pub fn mkdir_p(path: &str) -> bool;  // mkdir -p
pub fn rm_rf(path: &str) -> bool;    // rm -rf  
pub fn cp_r(src: &str, dest: &str) -> bool;  // cp -r
pub fn mv_file(src: &str, dest: &str) -> bool;  // mv
pub fn ln_s(target: &str, link: &str) -> bool;  // ln -s
pub fn chmod(path: &str, mode: &str) -> bool;   // chmod
pub fn chown(path: &str, owner: &str) -> bool;  // chown

// File content operations  
pub fn append_file(path: &str, content: &str) -> bool;  // >>
pub fn write_lines(path: &str, lines: &[String]) -> bool;
pub fn backup_file(path: &str, suffix: &str) -> String; // Create .bak file
pub fn rotate_file(path: &str, max_count: usize) -> bool; // log.1, log.2, etc

// Directory operations
pub fn list_files(path: &str, pattern: Option<&str>) -> Vec<String>;
pub fn find_files(path: &str, name_pattern: &str, recursive: bool) -> Vec<String>;
pub fn get_file_size(path: &str) -> u64;
pub fn get_file_mtime(path: &str) -> String;  // As ISO string
pub fn get_file_perms(path: &str) -> String;  // As octal string

// Usage with variable expansion
mkdir_p(&var!("$HOME/.config/$APP_NAME").expand());
backup_file(&config_file, &format!(".bak.{}", chrono::Utc::now().timestamp()));
```

### 5. Process & System Operations

**Missing system interaction:**
```rust
// Process operations (extend run_cmd)
pub fn run_cmd_silent(cmd: &str) -> bool;  // Discard output, return success
pub fn run_cmd_lines(cmd: &str) -> Vec<String>;  // Output as line vector
pub fn run_cmd_background(cmd: &str) -> u32;  // Return PID
pub fn kill_process(pid: u32, signal: Option<&str>) -> bool;
pub fn process_exists(pid: u32) -> bool;

// System info
pub fn get_user() -> String;        // $USER
pub fn get_home() -> String;        // $HOME  
pub fn get_pwd() -> String;         // $PWD
pub fn get_hostname() -> String;    // hostname
pub fn get_arch() -> String;        // uname -m
pub fn get_os() -> String;          // uname -s

// Environment manipulation
pub fn export_var(key: &str, value: &str);  // export KEY=value
pub fn unset_env(key: &str);                // unset KEY
pub fn source_file(path: &str);             // source file.sh (parse and load vars)

// Date/time operations
pub fn date_now(format: Option<&str>) -> String;  // date command
pub fn date_parse(date_str: &str, format: &str) -> Result<String, String>;
pub fn sleep_seconds(seconds: u64);
```

## Priority Implementation Order

### Phase 1: Critical String Operations (Week 1)
1. **Parameter expansion** - `param!` macro and var_* functions
2. **String tests** - `test!` macro and conditional functions  
3. **Advanced StringExt** - sed/awk-like operations

### Phase 2: File System Integration (Week 1.5)
4. **File operations** - mkdir_p, rm_rf, cp_r, etc.
5. **File content** - append, backup, rotate functions
6. **Directory traversal** - list_files, find_files

### Phase 3: System Integration (Week 2)
7. **Process operations** - background jobs, process management
8. **System info** - user, hostname, arch detection
9. **Environment** - export, source, environment manipulation

This gives you everything needed to port complex bash scripts while maintaining RSB's string-first philosophy and bash-like ergonomics.
