# RSB (Rebel String-Based) Rust Pattern + BashFX Integration

## Philosophy

RSB addresses a fundamental accessibility problem in Rust tooling: the gap between "academic correctness" and "getting stuff done." While Rust's type system provides incredible safety guarantees, its ceremonial complexity often creates barriers for developers who need to build practical CLI tools quickly.

RSB embraces **pragmatic simplicity** over theoretical purity. Instead of forcing developers to master complex generic signatures, trait bounds, and Result propagation chains, RSB provides a familiar, string-based interface that lets you focus on solving problems rather than appeasing the compiler.

RSB integrates core principles from the mature BashFX architecture, including function ordinality, XDG+ directory standards, sentinel-based operations, and the "Thisness" pattern for library context management.

## Why RSB Exists

### The "Academic Rust" Problem

Traditional Rust CLI development often looks like this:
```rust
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Build { target: Option<String> },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Build { target } => build_project(target)?,
    }
    Ok(())
}
```

This requires understanding: derive macros, trait systems, Result propagation, enum pattern matching, and Option handling - before you've written a single line of business logic.

### The RSB Alternative

```rust
fn main() {
    let args: Vec<String> = env::args().collect();
    rsb_bootstrap(&args);
    
    rsb_dispatch!(args, {
        "build" => do_build
    });
}

fn do_build(args: Args) -> i32 {
    let target = args.get_or(1, "debug");
    info!("Building target: {cyan}${}{reset}", target);
    0
}
```

No macros to learn, no trait bounds to satisfy, no Result chains to manage. Just familiar patterns that get out of your way.

## Design Principles

### Accessibility Over Purity
RSB chooses familiar patterns over idiomatic Rust when familiarity reduces cognitive load. Bash developers should feel at home immediately.

### Pragmatic Safety
Instead of preventing errors at compile-time through complex types, RSB prevents them at runtime through validation functions and clear error messages.

### Fail Fast, Fail Clear
When something goes wrong, RSB prints a helpful message and exits. No unwrapping Result chains or handling every possible error condition.

### String-First Design
Strings are the universal interface. They're what users type, what files contain, and what commands output. RSB embraces this reality instead of fighting it.

### Cognitive Load Reduction
Every RSB pattern should be learnable in minutes, not hours. If you need to consult documentation to understand basic usage, the design is wrong.

### Function Ordinality
Clear hierarchy prevents spaghetti code: Super-Ordinal → High-Order → Mid-Level → Low-Level. Each level has specific responsibilities and constraints.

### Self-Contained Ecosystem
Following BashFX principles: tools install cleanly, operate predictably, and can be completely removed without leaving traces.

## Core Principles

1. **String-First Design** - Everything is a string until proven otherwise
2. **No Heavy Dependencies** - Avoid clap, serde, and other "heavyweight" crates  
3. **Bash-Like Ergonomics** - `$1`, `$2`, variable expansion, simple dispatch
4. **Global Context** - Shared environment like shell variables
5. **Function Ordinality** - Clear hierarchy: Super → High → Mid → Low
6. **XDG+ Compliance** - Self-contained in `~/.local` with RSB/ODX namespacing
7. **Rewindable Operations** - Every action has a clear undo via sentinels
8. **Fail Fast, Simple Error Handling** - Print error and exit, no Result propagation chains
9. **BashFX Standard Interface** - Predictable variable naming and function patterns

## When to Use RSB

### ✅ Good Candidates
- Build scripts and automation tools
- Log processing utilities  
- Simple deployment scripts
- File manipulation tools
- Quick data transformation scripts
- Prototyping CLI tools
- Lightweight daemons and services
- Background monitoring tools
- Simple web servers and APIs
- **Converting legendary bash scripts into shareable tools**

### ❌ When RSB Isn't Right

RSB is explicitly **not** intended for:

- **Public libraries** where API stability and type safety are paramount
- **Performance-critical code** where zero-cost abstractions matter
- **Complex domain modeling** where the type system prevents errors
- **Large teams** where compile-time guarantees prevent integration bugs
- **Academic projects** where exploring Rust's type system is the goal

**RSB is for practitioners, not purists.** If you're building internal tools, automation scripts, or prototypes where developer velocity matters more than theoretical correctness, RSB is your friend.

## Mental Model

Think of RSB as **"Bash with Rust's runtime safety"** rather than **"Rust with simplified syntax."**

### The RSB Stack
```
User Input (strings) 
    ↓
Validation Functions (catch errors early)
    ↓  
Business Logic (string manipulation, file ops)
    ↓
Output (colored terminal messages, files)
```

### The Traditional Rust Stack
```
User Input
    ↓
Parse into Types (complex)
    ↓
Transform Types (generic)
    ↓
Handle Results (propagation)
    ↓
Serialize Output (complex)
```

RSB collapses this complexity into simple, linear workflows that match how humans think about problems.

## Getting Started

### For Bash Developers
If you're comfortable with bash scripting, RSB will feel familiar:

```bash
# Bash
PROJECT="my-app"
echo "Building $PROJECT"
```

```rust
// RSB
set_var("PROJECT", "my-app");
echo!("Building {blue}$PROJECT{reset}");
```

### For Rust Developers
If you're coming from traditional Rust, RSB might feel "wrong" at first. That's intentional. RSB trades compile-time guarantees for runtime simplicity:

**Traditional Rust:** Prevent errors through types
**RSB:** Catch errors through validation and fail clearly

**Traditional Rust:** Generic, reusable abstractions  
**RSB:** Specific, practical solutions

**Traditional Rust:** Perfect is the enemy of good
**RSB:** Good enough is usually perfect

## Core Components

### 1. Global Context with XDG+ Paths

The global context acts like shell environment variables with BashFX's XDG+ directory structure and RSB/ODX namespacing.

```rust
// RSB automatically sets up XDG+ hierarchy
rsb_bootstrap(&args);

// XDG+ Base Paths
// $XDG_HOME     = $HOME/.local
// $XDG_LIB      = $HOME/.local/lib  
// $XDG_ETC      = $HOME/.local/etc
// $XDG_BIN      = $HOME/.local/bin
// $XDG_DATA     = $HOME/.local/data

// RSB/ODX Namespaced Paths  
// $RSB_LIB      = $XDG_LIB/rsb
// $RSB_BIN      = $XDG_BIN/rsb  
// $RSB_ETC      = $XDG_ETC/rsb
// $ODX_LIB      = $XDG_LIB/odx (future)
// $ODX_BIN      = $XDG_BIN/odx (future)

// Tool-specific paths (via Thisness)
set_this_context("mytool", "$RSB_LIB/mytool", "$RSB_ETC/mytool.conf");
// $THIS_NAMESPACE = "mytool"
// $THIS_ROOT      = "$RSB_LIB/mytool" 
// $THIS_CONFIG    = "$RSB_ETC/mytool.conf"
// $THIS_BIN       = "$RSB_BIN/mytool"  (symlinked to $THIS_ROOT/bin/mytool)
// $THIS_LIB       = "$RSB_LIB/mytool"
```

### 2. BashFX Function Ordinality

RSB follows BashFX's strict function hierarchy for predictable call stack and responsibility separation.

```rust
// === SUPER-ORDINAL: Core Orchestrators ===
fn main() { /* Entry point, calls bootstrap + dispatch */ }

// === HIGH-ORDER: Independent Functions ===  
fn options(args: &mut Args) { /* Parse flags, set opt_* variables */ }
fn do_usage(args: Args) -> i32 { /* Show help, dispatchable */ }

// === HIGH-ORDER: Dispatchable Functions ===
fn do_build(args: Args) -> i32 { 
    // User-level validation and orchestration
    require_var!("PROJECT");
    let target = args.get_or(1, "debug");
    
    if _validate_build_target(&target) {
        __execute_build_command(&target)
    } else { 1 }
}

// === MID-LEVEL: Helpers ===
fn _validate_build_target(target: &str) -> bool {
    // Discrete sub-task, no user interaction
    ["debug", "release", "test"].contains(&target)
}

// === LOW-LEVEL: Literals ===  
fn __execute_build_command(target: &str) -> i32 {
    // "Close to metal" operation, trusts inputs
    run_cmd(&format!("cargo build --{}", target)).is_empty() as i32
}
```

**Ordinality Rules:**
- **High-Order**: Handle user input validation, apply user-level guards  
- **Mid-Level**: Perform discrete sub-tasks, no user interaction
- **Low-Level**: Trust inputs, only guard against system-level errors
- **Function calls flow DOWN the hierarchy only**

### 3. Enhanced Arguments with BashFX Patterns

RSB uses sophisticated argument parsing that handles flags, key-value pairs, and arrays while maintaining bash-style positional access.

```rust
fn do_build(mut args: Args) -> i32 {
    // BashFX-style predictable locals
    let mut ret = 1;                    // Status (always default to failure)
    let src = args.get_or(1, ".");      // Source path  
    let dest = args.get_or(2, "dist");  // Destination path
    let res: String;                    // Result value
    
    // Pop flags (removes from positional args)
    let opt_clean = args.has_pop("--clean");
    let opt_verbose = args.has_pop("--verbose");
    
    // Get flag values: --version 2.0.0 or --version=2.0.0
    let version = args.has_val("--version").unwrap_or_else(|| get_var("VERSION"));
    
    // Parse key-value: output=/tmp/build or features:logging,auth
    if let Some(output_dir) = args.get_kv("output") {
        set_var("BUILD_DIR", &output_dir);
    }
    
    // Parse arrays: features=logging,auth,metrics
    if let Some(features) = args.get_array("features") {
        set_array("BUILD_FEATURES", &features.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    }
    
    // After processing flags, positional args work normally
    let target = args.get_or(1, "debug");  // First unprocessed arg
    
    ret = 0; // Explicit success
    ret
}
```

**Supported Argument Formats:**
- **Flags**: `--clean`, `--verbose`, `-f`
- **Flag values**: `--file path.txt`, `--version=2.0.0`
- **Key-value**: `output=/tmp`, `config:production`
- **Arrays**: `features=auth,logging,metrics`
- **Positional**: Still work as `$1`, `$2` after flags are processed

### 4. RSB Stderr with BashFX Color Support

BashFX-inspired terminal output with color substitution and mode-based filtering.

```rust
// Mode-based output control (from environment)
// DEBUG_MODE    = enables info, warn, okay messages
// DEV_MODE      = enables developer-specific output  
// TRACE_MODE    = enables trace, think messages
// QUIET_MODE    = only error, fatal messages

// Color-aware messages with variable expansion
info!("Building {blue}$PROJECT{reset} v{yellow}$VERSION{reset}");
okay!("Build successful for {green}$target{reset}!");
warn!("Using default config at {grey}$config_path{reset}");
error!("Failed to read {red}$file_path{reset}");

// Debug helpers for complex data
debug_var("config", &my_config_struct);
trace_vars(&[("args", &args), ("result", &result)]);

// Mixed Rust formatting + RSB expansion
info!("Processing {} files in {cyan}$BUILD_DIR{reset}", file_count);
```

**Available Colors**: `{red}`, `{green}`, `{yellow}`, `{blue}`, `{grey}`, `{cyan}`, `{magenta}`, `{bold}`, `{reset}`

### 5. Sentinel-Based Rewindable Operations

BashFX's sentinel pattern for safe, reversible file modifications.

```rust
// Link RSB tool into user's shell profile
fn do_install(args: Args) -> i32 {
    let tool_name = get_var("SCRIPT_NAME");
    let sentinel = format!("RSB_{}", tool_name.to_uppercase());
    
    // Rewindable linking with sentinel
    link_with_sentinel(
        "$HOME/.bashrc",
        "source $RSB_ETC/rsb.rc",
        &sentinel
    );
    
    info!("Tool {blue}$tool_name{reset} installed with sentinel: {grey}$sentinel{reset}");
    0
}

fn do_uninstall(args: Args) -> i32 {
    let tool_name = get_var("SCRIPT_NAME");
    let sentinel = format!("RSB_{}", tool_name.to_uppercase());
    
    // Clean removal via sentinel
    unlink_with_sentinel("$HOME/.bashrc", &sentinel);
    info!("Tool {blue}$tool_name{reset} uninstalled");
    0
}
```

### 6. BashFX Thisness Pattern for Library Context

Enable generic library functions to adapt to any tool's context.

```rust
// Set up tool-specific context  
fn setup_tool_context() {
    set_this_context("mytool", "$RSB_LIB/mytool", "$RSB_ETC/mytool.conf");
}

// Generic library function that works for ANY RSB tool
pub fn lib_backup_config() -> i32 {
    require_var!("THIS_CONFIG");
    require_var!("THIS_NAMESPACE");
    
    let backup_path = var!("$THIS_LIB/backups/config-$(date +%s).bak");
    
    if is_file(&get_var("THIS_CONFIG")) {
        cat!(&get_var("THIS_CONFIG"))
            .to_file(&backup_path.expand());
        okay!("Backed up {cyan}$THIS_CONFIG{reset} to {grey}$backup_path{reset}");
        0
    } else {
        warn!("No config found at {yellow}$THIS_CONFIG{reset}");
        1
    }
}

// Tool-specific usage
fn do_backup(args: Args) -> i32 {
    lib_backup_config() // Automatically uses THIS tool's context
}
```

### 7. Stream Processing with BashFX Pipes

Powerful stream processing that mimics bash pipelines with type safety.

```rust
// File-based streaming
cat!("access.log")
    .grep("ERROR")
    .cut(4, " ")        // 4th field (1-indexed like bash)
    .sort()
    .uniq()
    .head(10)
    .each(|line| error!("Critical: {}", line));

// Command-based streaming  
cmd!("git log --oneline")
    .grep("fix")
    .tee("recent-fixes.txt")
    .count();

// Complex pipeline with variable expansion
cat!("$LOG_DIR/$APP_NAME.log")
    .filter(|line| !line.trim().is_empty())
    .grep(&var!("$ERROR_PATTERN").expand())
    .map(|line| line.to_uppercase())
    .to_file("$OUTPUT_DIR/errors.txt");
```

### 8. Configuration Management with Arrays

BashFX-style configuration with bash array support.

```rust
// Load multiple configs (last wins)
load_config!("/etc/myapp.conf", "$HOME/.myapprc", "./myapp.conf");

// Config file format supports:
// KEY=value
// KEY="value with spaces"  
// ARRAY=(item1 item2 item3)
// # comments

// Working with arrays
array!("TARGETS", ["debug", "release", "test"]);
let targets = get_array("TARGETS");
push_array("DEPLOY_HOSTS", "new-server.com");

// Save specific keys to file
save_config_file("./output.conf", &["PROJECT", "VERSION", "TARGETS"]);
```

### 9. Dual Dispatch System

RSB supports both pre-context (bootstrap) and context-aware command dispatch following BashFX patterns.

```rust
fn main() {
    let args: Vec<String> = env::args().collect();
    rsb_bootstrap(&args);
    
    // Pre-context dispatch for bootstrap commands (no config needed)
    if rsb_pre_dispatch!(args, {
        "install" => do_install,
        "uninstall" => do_uninstall,
        "init" => do_init,
        "check" => do_check
    }) {
        return; // Bootstrap command handled
    }
    
    // Load configuration after bootstrap
    load_config!("$THIS_CONFIG", "./tool.conf");
    
    // Main context-aware dispatch
    rsb_dispatch!(args, {
        "build" => do_build,
        "deploy" => do_deploy,
        "test" => do_test,
        "status" => do_status
    });
}
```

**Bootstrap Commands:** Don't depend on configuration or context - used for setup, installation, system checks.
**Context Commands:** Require loaded configuration and environment - main application functionality.

### 10. Type Checking and Validation

Built-in validation functions for common checks with helpful error messages.

```rust
// File system checks
require_file!("input.csv");   // Exits with error if not found
require_dir!("$BUILD_DIR");
require_command!("git");      // Exits if command not found  
require_var!("PROJECT");      // Exits if variable not set

// Conditional checks
if is_file("config.toml") { load_config!("config.toml"); }
if is_numeric("123.45") { /* process as number */ }
if is_name("my-project") { /* valid identifier */ }

// Custom validation with exit codes
validate!(is_command("docker"), "Docker is required for deployment", 2);
validate!(get_array("TARGETS").contains(&target.to_string()), 
          format!("Invalid target: {}. Valid: {}", target, get_array("TARGETS").join(", ")));
```

## Standard RSB Tool Structure

Following BashFX's script organization principles:

```rust
use rsb::*;

// === META (BashFX-style) ===
const TOOL_NAME: &str = "mytool";
const TOOL_VERSION: &str = "1.0.0";

// === BOOTSTRAP ===
fn main() {
    let args: Vec<String> = env::args().collect();
    rsb_bootstrap(&args);
    setup_tool_context();
    
    // Dual dispatch pattern
    if rsb_pre_dispatch!(args, {
        "install" => do_install,
        "uninstall" => do_uninstall,
        "reset" => do_reset,
        "check" => do_check
    }) { return; }
    
    load_config!("$THIS_CONFIG", "./mytool.conf");
    
    rsb_dispatch!(args, {
        "build" => do_build,
        "deploy" => do_deploy,
        "logs" => do_logs,
        "config" => do_config
    });
}

fn setup_tool_context() {
    set_var("TOOL_NAME", TOOL_NAME);
    set_var("TOOL_VERSION", TOOL_VERSION);
    set_this_context(TOOL_NAME, &var!("$RSB_LIB/$TOOL_NAME").expand(), &var!("$RSB_ETC/$TOOL_NAME.conf").expand());
}

// === HIGH-ORDER FUNCTIONS (Dispatchable) ===
fn do_build(mut args: Args) -> i32 {
    // BashFX predictable locals
    let mut ret = 1;
    let target = args.get_or(1, "debug");
    let opt_clean = args.has_pop("--clean");
    
    // User-level validation (High-Order responsibility)
    require_var!("PROJECT");
    let valid_targets = get_array("TARGETS");
    validate!(valid_targets.contains(&target.to_string()), 
              format!("Invalid target: {}. Valid: {}", target, valid_targets.join(", ")));
    
    // Delegate to helpers
    if opt_clean {
        _clean_workspace();
    }
    
    if _build_project(&# RSB (Rebel String-Based) Rust Pattern

## Philosophy

RSB is a design pattern for building Rust CLI tools that prioritizes simplicity and bash-like ergonomics over traditional Rust idioms. It's designed for the "too big for bash, too small for full Rust" sweet spot - tools that need more structure than shell scripts but don't warrant complex type systems and trait hierarchies.

## Core Principles

1. **String-First Design** - Everything is a string until proven otherwise
2. **No Heavy Dependencies** - Avoid clap, serde, and other "heavyweight" crates
3. **Bash-Like Ergonomics** - `$1`, `$2`, variable expansion, simple dispatch
4. **Global Context** - Shared environment like shell variables
5. **Fail Fast, Simple Error Handling** - Print error and exit, no Result propagation chains
6. **Pipeline Operations** - Chain string operations like bash pipes

## When to Use RSB

### ✅ Good Candidates
- Build scripts and automation tools
- Log processing utilities
- Simple deployment scripts
- File manipulation tools
- Quick data transformation scripts
- Prototyping CLI tools
- Lightweight daemons and services
- Background monitoring tools
- Simple web servers and APIs

### ❌ Not Suitable For
- Libraries intended for public consumption (use proper Rust patterns)
- Complex business logic with heavy state management
- Applications requiring strong typing guarantees
- High-frequency, performance-critical hot paths

## Core Components

### 1. Global Context (Environment Variables)

The global context acts like shell environment variables - a shared lookup table accessible from anywhere. All shell environment variables are automatically front-loaded at startup.

```rust
// All shell env vars are automatically loaded
// Setting variables
set_var("PROJECT", "my-app");
set_var("VERSION", "1.0.0");
set_var("BUILD_DIR", "/tmp/builds");

// Getting variables
let project = get_var("PROJECT");
let has_debug = has_var("DEBUG");

// Variable expansion
let path = var!("$BUILD_DIR/$PROJECT-v$VERSION");
let config = var!("${HOME}/.config/${PROJECT}/settings.toml");
```

### 2. Configuration File Handling

RSB supports loading and saving key-value configuration files with bash-style arrays.

```rust
// Load multiple configs (later files override earlier ones)
load_config!("/etc/myapp.conf", "$HOME/.myapprc", "./myapp.conf");

// Config file format supports:
// KEY=value
// KEY="value with spaces"
// ARRAY=(item1 item2 item3)
// # comments

// Working with arrays
array!("TARGETS", ["debug", "release", "test"]);
let targets = get_array("TARGETS");
push_array("DEPLOY_HOSTS", "new-server.com");

// Save specific keys to file
save_config_file("./output.conf", &["PROJECT", "VERSION", "TARGETS"]);
```

### 3. Stream Processing (Bash-like Pipes)

RSB provides powerful stream processing that mimics bash pipelines but with type safety.

```rust
// File-based streaming
cat!("access.log")
    .grep("ERROR")
    .cut(4, " ")
    .sort()
    .uniq()
    .head(10)
    .each(|line| println!("Error: {}", line));

// Command-based streaming  
cmd!("git log --oneline")
    .grep("fix")
    .tee("recent-fixes.txt")
    .count();

// String-based streaming
let data = "line1\nline2\nerror\nline3";
pipe!(data)
    .filter(|line| line.contains("error"))
    .map(|line| line.to_uppercase())
    .to_file("errors.txt");

// Complex pipeline with multiple sources
cat!("file1.txt", "file2.txt")
    .sed("old", "new")
    .grep("pattern")
    .sort()
    .to_string();
```

**Pattern Rules:**
- Use UPPERCASE for environment-style variables
- Set defaults early in main()
- Use descriptive names (`BUILD_DIR` not `DIR`)
- Check existence with `has_var()` before conditional logic

### 2. Enhanced Arguments Handling

RSB uses sophisticated argument parsing that handles flags, key-value pairs, and arrays while maintaining bash-style positional access.

```rust
fn build_project(mut args: Args) -> i32 {
    // Pop flags (removes from positional args)
    let clean = args.has_pop("--clean");
    let verbose = args.has_pop("--verbose");
    
    // Get flag values: --version 2.0.0 or --version=2.0.0
    let version = args.has_val("--version").unwrap_or_else(|| get_var("VERSION"));
    
    // Parse key-value: output=/tmp/build or features:logging,auth
    if let Some(output_dir) = args.get_kv("output") {
        set_var("BUILD_DIR", &output_dir);
    }
    
    // Parse arrays: features=logging,auth,metrics
    if let Some(features) = args.get_array("features") {
        set_array("BUILD_FEATURES", &features.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    }
    
    // After processing flags, positional args work normally
    let target = args.get_or(1, "debug");  // First unprocessed arg
    
    // Remaining unprocessed args
    let extra_args = args.remaining();
    
    0
}
```

**Supported Argument Formats:**
- **Flags**: `--clean`, `--verbose`, `-f`
- **Flag values**: `--file path.txt`, `--version=2.0.0`
- **Key-value**: `output=/tmp`, `config:production`
- **Arrays**: `features=auth,logging,metrics`
- **Positional**: Still work as `$1`, `$2` after flags are processed

### 3. Type Checking and Validation

Built-in validation functions for common checks with helpful error messages.

```rust
// File system checks
if is_file("config.toml") { /* ... */ }
require_file!("input.csv");  // Exits with error if not found
require_dir!("$BUILD_DIR");

// Value validation  
if is_numeric("123.45") { /* ... */ }
if is_name("my-project") { /* ... */ }  // Alphanumeric + _ -
validate!(is_command("git"), "Git is required");

// Variable requirements
require_var!("PROJECT");  // Exits if not set
require_command!("docker");  // Exits if command not found
```

**Available Validators:**
- `is_file()`, `is_dir()`, `is_entity()`, `is_link()`
- `is_string()`, `is_numeric()`, `is_empty()`, `is_name()`
- `is_command()`, `is_function()`
- `require_file!()`, `require_dir!()`, `require_var!()`, `require_command!()`

### 4. Dual Dispatch System

RSB supports both pre-context (bootstrap) and context-aware command dispatch.

```rust
fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Pre-context dispatch for bootstrap commands
    if rsb_pre_dispatch!(args, {
        "install" => install_deps,
        "init" => init_project,
        "check" => check_system
    }) {
        return; // Bootstrap command handled
    }
    
    // Load configuration after bootstrap
    load_config!("app.conf");
    
    // Main context-aware dispatch
    rsb_dispatch!(args, {
        "build" => build_project,
        "deploy" => deploy_app,
        "test" => run_tests
    });
}
```

**Bootstrap Commands:** Don't depend on configuration or context - used for setup, installation, system checks.

**Context Commands:** Require loaded configuration and environment - main application functionality.

### 5. Call Stack and Introspection

RSB automatically tracks function calls and provides introspection capabilities.

```rust
// Automatic introspection commands
// ./tool help       - Show all commands
// ./tool inspect    - List all functions with descriptions  
// ./tool stack      - Show current call stack

// Call stack is automatically maintained
fn deploy(args: Args) -> i32 {
    // Call stack: [main -> dispatch -> deploy]
    build_project(args);  // Stack: [main -> dispatch -> deploy -> build_project]
    0
}

// Manual call stack management
call_function("custom_task", &["arg1", "arg2"], || {
    // Function automatically added to call stack
    complex_operation();
});
```

### 3. String-Based Dispatch

Simple command routing without complex parsers.

```rust
fn main() {
    let args: Vec<String> = env::args().collect();
    
    rsb_dispatch!(args, {
        "build" => build_project,
        "deploy" => deploy,
        "logs" => show_logs,
        "config" => manage_config
    });
}
```

**Pattern Rules:**
- Keep command names simple (single words)
- Use verbs for actions (`build`, `deploy`, `clean`)
- Use nouns for data operations (`config`, `logs`, `status`)
- Fall back to help for unknown commands

### 4. Variable Expansion

Bash-style variable substitution in strings.

```rust
// Both syntaxes work
let simple = var!("$HOME/.config");
let braced = var!("${PROJECT}_${VERSION}.tar.gz");

// In templates
let template = "Building $1 for $PROJECT in $BUILD_MODE";
let expanded = args.expand(template);

// Conditional expansion (if var exists)
if has_var("VERBOSE") {
    let debug_cmd = var!("cargo build --verbose");
}
```

**Pattern Rules:**
- Use `$VAR` for simple cases
- Use `${VAR}` when adjacent to other text
- Set variables before using them
- Use meaningful variable names

### 5. String Pipeline Operations

Chain operations like bash pipes.

```rust
let content = read_file("app.log");
let errors = content
    .grep("ERROR")
    .head(10)
    .join("\n");

// Or as a pipeline
let result = read_file("data.csv")
    .cut(2, ",")        // Get 3rd column
    .grep("production") // Filter lines
    .tail(5);           // Last 5 entries
```

**Pattern Rules:**
- Chain operations for readability
- Each operation returns a new string or Vec<String>
- Keep operations simple and composable
- Use descriptive method names

## Error Handling Philosophy

RSB favors "fail fast" over complex error propagation.

```rust
// ✅ RSB Style
fn read_config(path: &str) -> String {
    read_file(path).unwrap_or_else(|_| {
        eprintln!("Failed to read config: {}", path);
        std::process::exit(1);
    })
}

// ❌ Traditional Rust
fn read_config(path: &str) -> Result<Config, Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}
```

**Error Handling Rules:**
- Print meaningful error messages
- Exit immediately on unrecoverable errors
- Use `unwrap_or_default()` for optional values
- Only use `Result` when the caller needs to handle errors

## File Operations

Keep file operations simple and string-focused.

```rust
// Reading
let content = read_file("config.toml");
let lines = content.lines().collect::<Vec<_>>();

// Writing
write_file("output.txt", &processed_data);

// Command execution
let output = run_cmd("git status --porcelain");
let files = output.lines().map(|l| l.trim()).collect();
```

**File Operation Rules:**
- Return strings, not complex types
- Handle common cases (file not found) gracefully
- Use command execution for complex operations
- Work with lines as the basic unit

## Common Patterns

### Configuration Management

```rust
fn load_config() {
    // Try multiple sources
    let config_paths = vec![
        var!("$HOME/.config/$PROJECT/config.toml"),
        var!("/etc/$PROJECT/config.toml"),
        "./config.toml".to_string()
    ];
    
    for path in config_paths {
        if std::path::Path::new(&path.expand()).exists() {
            let content = read_file(&path.expand());
            parse_simple_config(&content);
            break;
        }
    }
}

fn parse_simple_config(content: &str) {
    for line in content.lines() {
        if let Some((key, value)) = line.split_once('=') {
            set_var(key.trim(), value.trim());
        }
    }
}
```

### Conditional Execution

```rust
fn build_project(args: Args) {
    if args.has("--clean") {
        run_cmd("cargo clean");
    }
    
    let profile = if args.has("--release") { "release" } else { "debug" };
    set_var("BUILD_PROFILE", profile);
    
    let build_cmd = var!("cargo build ${BUILD_PROFILE:+--}${BUILD_PROFILE}");
    run_cmd(&build_cmd.expand());
}
```

### Data Processing

```rust
fn process_logs(args: Args) {
    let log_file = args.get_or(0, "app.log");
    let content = read_file(log_file);
    
    if args.has("--errors") {
        let errors = content
            .grep("ERROR")
            .head(args.get_or(1, "20").parse().unwrap_or(20));
        
        for error in errors {
            println!("{}", error);
        }
    }
}
```

## Best Practices

### Initialization

```rust
fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Front-load ALL environment variables into RSB context
    for (key, value) in std::env::vars() {
        set_var(&key, &value);
    }
    
    // Set up $0 and path awareness (override env if needed)
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
    
    // Override PWD with current directory
    set_var("PWD", env::current_dir().unwrap().to_string_lossy());
    
    // Set defaults for missing common vars
    if !has_var("BUILD_ENV") {
        set_var("BUILD_ENV", "development");
    }
    
    // Load config if exists
    if std::path::Path::new("rsb.conf").exists() {
        load_config();
    }
    
    dispatch(args);
}
```

### Function Organization

```rust
// ✅ Good: Simple, focused functions
fn build_project(args: Args) { /* ... */ }
fn deploy_app(args: Args) { /* ... */ }
fn show_status(args: Args) { /* ... */ }

// ❌ Avoid: Complex parameter objects
fn build_project(config: BuildConfig, options: BuildOptions) { /* ... */ }
```

### Variable Naming

```rust
// ✅ Environment-style variables
set_var("PROJECT_NAME", "my-app");
set_var("BUILD_DIR", "/tmp/builds");
set_var("DEPLOY_HOST", "server.com");

// ✅ Descriptive locals
let source_file = args.get_or(0, "input.txt");
let target_env = args.get_or(1, "staging");

// ❌ Avoid cryptic names
set_var("P", "my-app");
let f = args.get(0);
```

## Common Anti-Patterns

### Over-Engineering

```rust
// ❌ Don't do this in RSB
#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    config: Option<PathBuf>,
    #[arg(short, long)]
    verbose: bool,
}

// ✅ RSB way
fn main() {
    let args: Vec<String> = env::args().collect();
    let config_file = args.get_or(1, "config.toml");
    let verbose = args.has("--verbose");
}
```

### Complex Error Handling

```rust
// ❌ Avoid complex Result chains
fn process() -> Result<(), Box<dyn Error>> {
    let config = load_config()?;
    let data = fetch_data(&config)?;
    let result = transform_data(data)?;
    save_result(result)?;
    Ok(())
}

// ✅ RSB style
fn process(args: Args) {
    let config_file = args.get_or(0, "config.toml");
    load_config(&config_file);
    
    let data = fetch_data();
    let result = transform_data(&data);
    save_result(&result);
    
    println!("Processing complete");
}
```

### Type Over-Specification

```rust
// ❌ Avoid heavy typing
struct Config {
    database_url: Url,
    port: u16,
    features: HashSet<Feature>,
}

// ✅ Keep it simple
fn load_config() {
    set_var("DATABASE_URL", "postgres://localhost:5432/db");
    set_var("PORT", "3000");
    set_var("FEATURES", "auth,logging,metrics");
}
```

## Integration with Traditional Rust

RSB tools can still use proper Rust patterns where it makes sense:

```rust
// Use RSB for CLI interface
fn main() {
    let args: Vec<String> = env::args().collect();
    rsb_dispatch!(args, {
        "analyze" => analyze_data,
        "report" => generate_report
    });
}

// Use traditional Rust for complex logic
fn analyze_data(args: Args) {
    let input_file = args.get_or(0, "data.csv");
    let content = read_file(&input_file);
    
    // Switch to proper Rust for heavy lifting
    let analysis_result = heavy_analysis_module::process(&content);
    
    // Back to RSB for output
    let output_file = var!("${input_file%.csv}_analysis.txt");
    write_file(&output_file.expand(), &analysis_result.summary);
}
```

### Error Codes and Return Values

RSB supports bash-style return codes for functions:

```rust
// Function that can return different exit codes
fn deploy_app(args: Args) -> i32 {
    let env = args.get_or(1, "staging");
    
    if !has_var("VERSION") {
        eprintln!("No version set!");
        return 1;  // Error
    }
    
    if env == "production" && !args.has("--confirmed") {
        eprintln!("Production deploy requires --confirmed flag");
        return 2;  // Different error
    }
    
    let result = run_cmd_with_status(&var!("deploy.sh $env").expand());
    match result.status {
        0 => {
            println!("Deploy successful: {}", result.output);
            0  // Success
        },
        _ => {
            eprintln!("Deploy failed: {}", result.error);
            result.status
        }
    }
}

// Enhanced command execution with status
pub struct CmdResult {
    pub status: i32,
    pub output: String,  // stdout
    pub error: String,   // stderr
}

pub fn run_cmd_with_status(cmd: &str) -> CmdResult {
    // Implementation that captures both output and status
}
```

### Function Return Types

RSB functions can return different types based on use case:

```rust
// Simple functions - no return (assume success)
fn clean_workspace(args: Args) {
    run_cmd("cargo clean");
    println!("Workspace cleaned");
}

// Functions with status codes
fn validate_config(args: Args) -> i32 {
    if !std::path::Path::new("config.toml").exists() {
        return 1;
    }
    0
}

// Functions that return strings (for capture/piping)
fn get_version(args: Args) -> String {
    run_cmd("git describe --tags").trim().to_string()
}

// Functions that return structured results
fn build_project(args: Args) -> CmdResult {
    let output = run_cmd_with_status("cargo build");
    if output.status == 0 {
        set_var("LAST_BUILD", "success");
    }
    output
}
```

## RSB Implementation Details

Understanding the Rust patterns that make RSB work:

### Core Rust Patterns Used

#### 1. Lazy Static for Global State
```rust
// Global context using lazy_static
lazy_static::lazy_static! {
    pub static ref CTX: Arc<Mutex<Context>> = Arc::new(Mutex::new(Context::new()));
}
```
**Why**: Provides thread-safe global state that initializes on first access, mimicking shell environment variables.

#### 2. Trait Extensions for String Methods
```rust
pub trait StringExt {
    fn grep(&self, pattern: &str) -> Vec<String>;
    fn cut(&self, field: usize, delimiter: &str) -> String;
}

impl StringExt for String {
    // Implementations that add bash-like methods to String
}
```
**Why**: Extends standard types with domain-specific methods without inheritance.

#### 3. Declarative Macros for DSL
```rust
#[macro_export]
macro_rules! rsb_dispatch {
    ($args:expr, { $($cmd:literal => $handler:ident),* }) => {
        // Pattern matching code generation
    }
}

#[macro_export] 
macro_rules! var {
    ($text:expr) => { Var::new($text) };
}
```
**Why**: Creates domain-specific syntax that feels natural while generating efficient Rust code.

#### 4. Newtype Pattern for Type Safety
```rust
pub struct Var(String);  // Wraps String with expansion behavior
pub struct Args<'a>(&'a [String]);  // Wraps slice with bash-like methods
```
**Why**: Adds behavior to existing types without performance overhead.

#### 5. Builder Pattern for Complex Operations
```rust
impl StringExt for String {
    fn grep(&self, pattern: &str) -> Vec<String> { /* ... */ }
}
// Allows chaining: content.grep("ERROR").head(10).join("\n")
```
**Why**: Enables fluent, pipeline-like syntax similar to bash pipes.

#### 6. Function Pointers for Dynamic Dispatch
```rust
type Handler = fn(&Args) -> i32;
// Or for different return types:
type SimpleHandler = fn(Args);
type StatusHandler = fn(Args) -> i32; 
type OutputHandler = fn(Args) -> String;
```
**Why**: Allows runtime dispatch while maintaining zero-cost abstractions.

#### 7. Regex for Variable Expansion
```rust
use regex::Regex;
let var_re = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap();
```
**Why**: Handles complex variable substitution patterns safely and efficiently.

#### 8. Arc<Mutex<T>> for Shared Mutable State
```rust
pub static ref CTX: Arc<Mutex<Context>> = // ...
```
**Why**: Thread-safe shared state that can be modified from anywhere, like shell variables.

### Key Structs and Their Roles

#### Context Struct
```rust
pub struct Context {
    vars: HashMap<String, String>,  // Variable storage
}
```
**Purpose**: Central store for all RSB variables, mimics shell environment.

#### Args Struct  
```rust
pub struct Args<'a> {
    args: &'a [String],  // Borrowed slice for zero-copy
}
```
**Purpose**: Provides bash-like argument access patterns ($1, $2, etc.).

#### Var Struct
```rust
pub struct Var(String);  // Newtype with automatic expansion
```
**Purpose**: Strings that automatically expand variables when created.

#### CmdResult Struct
```rust
pub struct CmdResult {
    pub status: i32,
    pub output: String,
    pub error: String,
}
```
**Purpose**: Captures full command execution state like bash $?, stdout, stderr.

### Macro Implementations

#### Variable Expansion Macro
```rust
macro_rules! var {
    ($text:expr) => {
        {
            let expanded = CTX.lock().unwrap().expand($text);
            Var(expanded)
        }
    };
}
```

#### Dispatch Macro
```rust
macro_rules! rsb_dispatch {
    ($args:expr, { $($cmd:literal => $handler:ident),* }) => {
        let command = $args.get(1).unwrap_or("help");
        let cmd_args = Args::new(&$args[2..]);  // Strip program name and command
        
        match command {
            $($cmd => {
                let result = $handler(cmd_args);
                std::process::exit(result);  // Handle return codes
            },)*
            _ => {
                eprintln!("Unknown command: {}", command);
                std::process::exit(1);
            }
        }
    }
}
```

### Memory and Performance Characteristics

- **Zero-copy argument handling**: Args borrows from original Vec<String>
- **Lazy variable expansion**: Only expands when needed via var! macro
- **Minimal allocations**: String operations return new strings only when necessary
- **Thread safety**: Global context protected by Arc<Mutex<T>>
- **Compile-time dispatch**: Macros generate direct function calls

### Integration Points

RSB integrates with standard Rust through:

1. **std::process** for command execution
2. **std::fs** for file operations  
3. **std::env** for environment variables
4. **regex** crate for pattern matching
5. **lazy_static** for global state

The pattern stays within Rust's safety guarantees while providing bash-like ergonomics.

## RSB_ASSETS: Complete Implementation Reference

### Macros

#### `rsb_dispatch!`
```rust
#[macro_export]
macro_rules! rsb_dispatch {
    ($args:expr, { $($cmd:literal => $handler:ident),* }) => { /* ... */ }
}
```
**Purpose**: Main command dispatcher, generates match statement for routing commands to handlers.
**Usage**: `rsb_dispatch!(args, { "build" => build_fn, "deploy" => deploy_fn });`

#### `var!`
```rust
#[macro_export]
macro_rules! var {
    ($text:expr) => { Var::new($text) };
}
```
**Purpose**: Creates variable-expanded strings from templates.
**Usage**: `var!("$HOME/.config/$APP_NAME/settings.toml")`

#### `echo!`
```rust
#[macro_export]
macro_rules! echo {
    ($text:expr) => { println!("{}", var!($text).expand()); };
    ($text:expr, $($args:expr),*) => { println!("{}", format!($text, $($args),*)); };
}
```
**Purpose**: Bash-like echo with automatic variable expansion.
**Usage**: `echo!("Building $PROJECT v$VERSION");`

#### Enhanced Args Struct
```rust
pub struct Args {
    args: Vec<String>,
    processed: std::collections::HashSet<usize>,
}
```
**Purpose**: Sophisticated argument parsing that tracks processed flags/options.
**Key Methods**: `has_pop()`, `has_val()`, `get_kv()`, `get_array()`, `remaining()`

#### Validation Functions
```rust
pub fn is_file(path: &str) -> bool
pub fn is_dir(path: &str) -> bool  
pub fn is_entity(path: &str) -> bool
pub fn is_link(path: &str) -> bool
pub fn is_string(value: &str) -> bool
pub fn is_numeric(value: &str) -> bool
pub fn is_empty(value: &str) -> bool
pub fn is_name(value: &str) -> bool
pub fn is_command(cmd: &str) -> bool
pub fn is_function(name: &str) -> bool
```
**Purpose**: Type checking and validation with variable expansion support.

#### Call Stack Management
```rust
pub struct CallFrame {
    pub function: String,
    pub args: Vec<String>, 
    pub timestamp: std::time::SystemTime,
    pub context_snapshot: HashMap<String, String>,
}
```
**Purpose**: Track function calls with full context for debugging and introspection.

### Macros

#### Enhanced Dispatch Macros
```rust
#[macro_export]
macro_rules! rsb_pre_dispatch { /* ... */ }

#[macro_export] 
macro_rules! rsb_dispatch { /* ... */ }
```
**Purpose**: Two-phase dispatch system - bootstrap commands vs context-aware commands.
**Features**: Automatic function registration, call stack management, built-in help/inspect.

#### Validation Macros
```rust
#[macro_export]
macro_rules! validate { /* ... */ }

#[macro_export]
macro_rules! require_file { /* ... */ }
#[macro_export]
macro_rules! require_dir { /* ... */ }
#[macro_export]
macro_rules! require_command { /* ... */ }
#[macro_export]
macro_rules! require_var { /* ... */ }
```
**Purpose**: Declarative validation with automatic error handling and exit codes.

### Functions

#### Introspection Functions
```rust
pub fn register_function(name: &str, description: &str)
pub fn list_functions() -> Vec<(String, String)>
pub fn show_help()
pub fn show_functions() 
pub fn show_call_stack()
```
**Purpose**: Runtime introspection and help generation.

#### Call Stack Functions
```rust
pub fn push_call(function: &str, args: &[String])
pub fn pop_call() -> Option<CallFrame>
pub fn get_call_stack() -> Vec<CallFrame>
pub fn call_function<F, R>(name: &str, args: &[String], func: F) -> R
```
**Purpose**: Manual call stack management for complex control flow.

### Traits

#### `StringExt`
```rust
pub trait StringExt {
    fn cut(&self, field: usize, delimiter: &str) -> String;
    fn grep(&self, pattern: &str) -> Vec<String>;
    fn sed(&self, from: &str, to: &str) -> String;
    fn head(&self, n: usize) -> Vec<String>;
    fn tail(&self, n: usize) -> Vec<String>;
    fn trim_lines(&self) -> String;
}
```
**Purpose**: Extends String with bash-like text processing methods.
**Pattern**: Extension trait for adding domain-specific methods to existing types.

### Functions

#### Global Context Functions
```rust
pub fn set_var<K: Into<String>, V: Into<String>>(key: K, value: V)
pub fn get_var(key: &str) -> String
pub fn has_var(key: &str) -> bool
pub fn unset_var(key: &str)
pub fn expand_vars(text: &str) -> String
```
**Purpose**: Global variable manipulation, thread-safe wrappers around Context.

#### File Operations
```rust
pub fn read_file(path: &str) -> String
pub fn write_file(path: &str, content: &str)
```
**Purpose**: Simple file I/O that returns/accepts strings, handles errors internally.

#### Configuration Functions
```rust
pub fn load_config_file(path: &str)
pub fn parse_config_content(content: &str)
pub fn save_config_file(path: &str, keys: &[&str])
```
**Purpose**: Load/save key-value config files with bash-style arrays.

#### Array Functions
```rust
pub fn set_array(key: &str, items: &[&str])
pub fn get_array(key: &str) -> Vec<String>
pub fn push_array(key: &str, item: &str)
```
**Purpose**: Bash-style array manipulation in global context.

#### Stream Factory Functions
```rust
pub fn grep_fn(pattern: &str) -> impl Fn(Stream) -> Stream
pub fn sed_fn(from: &str, to: &str) -> impl Fn(Stream) -> Stream  
pub fn head_fn(n: usize) -> impl Fn(Stream) -> Stream
pub fn tail_fn(n: usize) -> impl Fn(Stream) -> Stream
pub fn cut_fn(field: usize, delimiter: &str) -> impl Fn(Stream) -> Stream
```
**Purpose**: Higher-order functions for building reusable stream operations.

### Type Aliases

#### Handler Function Types
```rust
type SimpleHandler = fn(Args);
type StatusHandler = fn(Args) -> i32;
type OutputHandler = fn(Args) -> String;
type ResultHandler = fn(Args) -> CmdResult;
```
**Purpose**: Function pointer types for different handler patterns in dispatch.

### Global Statics

#### `CTX`
```rust
lazy_static::lazy_static! {
    pub static ref CTX: Arc<Mutex<Context>> = Arc::new(Mutex::new(Context::new()));
}
```
**Purpose**: Thread-safe global context for variables, initialized on first access.
**Pattern**: Lazy static singleton with interior mutability.

### Standard Library Usage

#### `std::collections::HashMap`
**Usage**: Variable storage in Context struct.
**Why**: Fast key-value lookup for environment-style variables.

#### `std::sync::{Arc, Mutex}`
**Usage**: Thread-safe global context.
**Why**: Allows shared mutable state across threads safely.

#### `std::env`
**Usage**: Reading initial environment variables, command line args.
**Why**: Interface to system environment.

#### `std::process::Command`
**Usage**: Shell command execution.
**Why**: Safe subprocess management with output capture.

#### `std::fs`
**Usage**: File read/write operations.
**Why**: Standard file system interface.

#### `std::path::Path`
**Usage**: Path manipulation for script awareness.
**Why**: Cross-platform path handling.

### External Dependencies

#### `lazy_static`
**Usage**: Global context initialization.
**Why**: Provides safe global state without runtime initialization overhead.

#### `regex`
**Usage**: Variable expansion pattern matching (`$VAR`, `${VAR}`).
**Why**: Robust pattern matching for bash-style variable substitution.

### Rust Patterns Employed

#### **Newtype Pattern**
- `Var(String)` - Adds behavior to String
- `Args(&[String])` - Adds bash-like methods to string slice

#### **Extension Trait Pattern**
- `StringExt` - Adds methods to existing types without inheritance

#### **Builder Pattern**
- Method chaining: `content.grep("ERROR").head(10).join("\n")`

#### **Zero-Cost Abstractions**
- `Args` borrows data, no copying
- Macros generate efficient code at compile time

#### **Interior Mutability Pattern**
- `Arc<Mutex<Context>>` - Immutable reference to mutable data

#### **RAII (Resource Acquisition Is Initialization)**
- File operations handle cleanup automatically
- Command execution manages process lifecycle

#### **Type State Pattern**
- Different handler types enforce correct usage at compile time

#### **Compile-Time Code Generation**
- Macros generate match statements and function calls

#### **Fail-Fast Error Handling**
- Functions exit on error rather than propagating `Result`s
- Follows Unix philosophy of "do one thing well"

### Memory Management

#### **Stack Allocation**
- `Args` uses borrowed references, no heap allocation
- `Var` expands once and stores result

#### **Heap Usage**
- `HashMap` for variable storage
- `String` for variable values and command output
- `Vec<String>` for line-based operations

#### **Reference Counting**
- `Arc<Context>` for shared ownership of global state

### Performance Characteristics

#### **Lazy Evaluation**
- Variables only expanded when accessed via `var!` or `expand()`
- Global context initialized on first use

#### **Zero-Copy Where Possible**
- `Args` borrows from original `Vec<String>`
- String slicing instead of allocation where feasible

#### **Minimal Runtime Overhead**
- Macros generate direct function calls
- No dynamic dispatch unless explicitly needed

#### **Thread Safety**
- All global operations protected by mutex
- Lock contention only on variable access

This comprehensive reference shows how RSB achieves bash-like ergonomics while maintaining Rust's safety and performance characteristics through careful use of Rust's type system, ownership model, and zero-cost abstractions.

## Bash to RSB Pattern Mapping

RSB translates familiar bash patterns into Rust equivalents. Here's how common bash constructs map to RSB:

### Variable Assignment and Expansion

```bash
# Bash
PROJECT="my-app"
VERSION="1.0.0"
BUILD_PATH="/tmp/builds/$PROJECT-v$VERSION"
echo "Building $PROJECT in ${BUILD_PATH}"
```

```rust
// RSB
set_var("PROJECT", "my-app");
set_var("VERSION", "1.0.0");
let build_path = var!("/tmp/builds/$PROJECT-v$VERSION");
println!("Building {} in {}", get_var("PROJECT"), build_path);
```

### Positional Arguments

```bash
# Bash function
build() {
    local target=${1:-debug}
    local version=${2:-1.0.0}
    echo "Building $target version $version"
    echo "All args: $@"
    echo "Arg count: $#"
}
```

```rust
// RSB function
fn build(args: Args) {
    let target = args.get_or(1, "debug");    // $1
    let version = args.get_or(2, "1.0.0");   // $2
    println!("Building {} version {}", target, version);
    println!("Script: {}", args.get(0));     // $0 - script name
    println!("All args: {}", args.join(" "));
    println!("Arg count: {}", args.len());
}
```

### Command Dispatch

```bash
# Bash
case "$1" in
    build)
        build_project "$2" "$3"
        ;;
    deploy)
        deploy_app "$2" "$3"
        ;;
    clean)
        clean_workspace
        ;;
    *)
        echo "Unknown command: $1"
        exit 1
        ;;
esac
```

```rust
// RSB - dispatch automatically strips command and passes remaining args
rsb_dispatch!(args, {
    "build" => build_project,    // Gets args[2..] as Args
    "deploy" => deploy_app,      // Gets args[2..] as Args  
    "clean" => clean_workspace   // Gets args[2..] as Args
});

// The build_project function receives only the arguments after "build"
// If called as: ./tool build debug --clean
// build_project gets: Args(["debug", "--clean"])
```

### Flag Checking

```bash
# Bash
if [[ "$*" == *"--verbose"* ]]; then
    VERBOSE=true
fi

if [[ "$*" == *"--force"* ]]; then
    rm -rf /dangerous/path
fi
```

```rust
// RSB
if args.has("--verbose") {
    set_var("VERBOSE", "true");
}

if args.has("--force") {
    run_cmd("rm -rf /dangerous/path");
}
```

### File Operations

```bash
# Bash
content=$(cat config.txt)
echo "$content" | grep "ERROR" | head -10 > errors.txt
lines=$(wc -l < data.txt)
```

```rust
// RSB
let content = read_file("config.txt");
let errors = content
    .grep("ERROR")
    .head(10)
    .join("\n");
write_file("errors.txt", &errors);

let lines = read_file("data.txt").lines().count();
```

### Command Execution

```bash
# Bash
output=$(git status --porcelain)
if [ $? -eq 0 ]; then
    echo "Git status: $output"
else
    echo "Git command failed"
    exit 1
fi
```

```rust
// RSB
let output = run_cmd("git status --porcelain");
if !output.is_empty() {
    println!("Git status: {}", output);
} else {
    eprintln!("Git command failed");
    std::process::exit(1);
}
```

### String Processing

```bash
# Bash
echo "$data" | cut -d',' -f2 | sort | uniq
filename=$(basename "$path" .txt)
extension="${filename##*.}"
```

```rust
// RSB
let result = data
    .cut(2, ",")  // 2nd field (1-indexed like bash)
    .lines()
    .map(|s| s.to_string())
    .collect::<std::collections::HashSet<_>>()
    .into_iter()
    .sorted()
    .collect::<Vec<_>>()
    .join("\n");

let filename = std::path::Path::new(&path)
    .file_stem()
    .unwrap()
    .to_str()
    .unwrap();
```

### Flag and Value Parsing

```bash
# Bash
while [[ $# -gt 0 ]]; do
    case $1 in
        --clean)
            CLEAN=true
            shift
            ;;
        --version=*)
            VERSION="${1#*=}"
            shift
            ;;
        --file)
            INPUT_FILE="$2"
            shift 2
            ;;
        *)
            ARGS+=("$1")
            shift
            ;;
    esac
done
```

```rust
// RSB - much simpler!
let clean = args.has_pop("--clean");
let version = args.has_val("--version").unwrap_or("1.0.0".to_string());
let input_file = args.has_val("--file");

// Key-value parsing
let output_dir = args.get_kv("output");  // output=/tmp or output:/tmp
let features = args.get_array("features");  // features=auth,logging,metrics

// Remaining positional args still work
let target = args.get_or(1, "debug");
```

### System Validation

```bash
# Bash
check_command() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "Error: $1 is required but not installed."
        exit 1
    fi
}

check_file() {
    if [ ! -f "$1" ]; then
        echo "Error: File $1 not found."
        exit 1
    fi
}

check_command git
check_command docker
check_file config.toml
```

```rust
// RSB - built-in validation
require_command!("git");
require_command!("docker");  
require_file!("config.toml");

// Or conditional checks
if !is_command("docker") {
    echo!("Warning: Docker not found, some features disabled");
    set_var("DOCKER_ENABLED", "false");
}

validate!(is_numeric(&version), "Version must be numeric");
validate!(is_name(&project_name), "Invalid project name");
```

### Dual Dispatch Pattern

```bash
# Bash - typically one big case statement
case "$1" in
    install|init|check)
        # Bootstrap commands
        ;;
    build|deploy|test)
        # Load config first
        source config.sh
        # Main commands
        ;;
esac
```

```rust
// RSB - clean separation
// Bootstrap first (no config needed)
if rsb_pre_dispatch!(args, {
    "install" => install_deps,
    "init" => init_project,
    "check" => check_system
}) {
    return;
}

// Then load config and run main commands
load_config!("app.conf");
rsb_dispatch!(args, {
    "build" => build_project,
    "deploy" => deploy_app
});
```

### Loops and Iteration

```bash
# Bash
for file in *.txt; do
    process_file "$file"
done

while IFS= read -r line; do
    echo "Processing: $line"
done < input.txt

# Process array
HOSTS=(web1 web2 db1)
for host in "${HOSTS[@]}"; do
    ping "$host"
done
```

```rust
// RSB
let files = run_cmd("ls *.txt");
for file in files.lines() {
    process_file(file.trim());
}

cat!("input.txt")
    .each(|line| echo!("Processing: $line"));

// Process array
array!("HOSTS", ["web1", "web2", "db1"]);
let hosts = get_array("HOSTS");
for host in hosts {
    run_cmd(&format!("ping {}", host));
}
```

### Stream Processing Pipelines

```bash
# Bash
cat access.log | grep "ERROR" | cut -d' ' -f4 | sort | uniq -c | head -10 > errors.txt
tail -f app.log | grep "WARN" | tee warnings.txt
find . -name "*.rs" | xargs wc -l | sort -n
```

```rust
// RSB
cat!("access.log")
    .grep("ERROR")
    .cut(4, " ")
    .sort()
    .uniq()
    .head(10)
    .to_file("errors.txt");

cmd!("tail -f app.log")
    .grep("WARN")
    .tee("warnings.txt");

cmd!("find . -name '*.rs'")
    .map(|file| format!("{} {}", run_cmd(&format!("wc -l {}", file)).trim(), file))
    .sort()
    .each(|line| println!("{}", line));
```

### Configuration Files

```bash
# Bash
# Load config file
source config.sh

# config.sh content:
PROJECT="my-app"
VERSION="1.0.0"
DEPLOY_HOSTS=(web1.com web2.com db.com)
BUILD_FLAGS="--release --target x86_64"

# Use arrays
for host in "${DEPLOY_HOSTS[@]}"; do
    echo "Deploying to $host"
done
```

```rust
// RSB
load_config!("config.conf");

// config.conf content:
// PROJECT=my-app
// VERSION=1.0.0
// DEPLOY_HOSTS=(web1.com web2.com db.com)
// BUILD_FLAGS=--release --target x86_64

// Use arrays
let hosts = get_array("DEPLOY_HOSTS");
for host in hosts {
    echo!("Deploying to $host");
}

// Or using streams
get_array("DEPLOY_HOSTS")
    .iter()
    .for_each(|host| echo!("Deploying to $host"));
```

### Environment Variables

```bash
# Bash
export BUILD_ENV=${BUILD_ENV:-development}
export PATH="$HOME/bin:$PATH"
echo "User: $USER, Home: $HOME"
```

```rust
// RSB - all env vars are front-loaded into context
// No direct std::env::var calls needed
let BUILD_ENV = get_var("BUILD_ENV");  // Already loaded from shell
let PATH = var!("$HOME/bin:$PATH");    // Use same var names as bash
set_var("PATH", &PATH.expand());       // Update the context

println!("User: {}, Home: {}", get_var("USER"), get_var("HOME"));
```

### Complex String Templates

```bash
# Bash
cat > deploy.sh << EOF
#!/bin/bash
ssh $DEPLOY_USER@$DEPLOY_HOST "
    cd /opt/$PROJECT &&
    docker pull $IMAGE:$VERSION &&
    docker-compose up -d
"
EOF
```

```rust
// RSB
let deploy_script = var!(r#"#!/bin/bash
ssh $DEPLOY_USER@$DEPLOY_HOST "
    cd /opt/$PROJECT &&
    docker pull $IMAGE:$VERSION &&
    docker-compose up -d
"
"#);
write_file("deploy.sh", &deploy_script.expand());

// Or using echo! macro
echo!("Deploying $PROJECT v$VERSION to $DEPLOY_HOST");
echo!("Script saved to: $PWD/deploy.sh");
```

### Error Handling

```bash
# Bash
set -e  # Exit on error
command_that_might_fail || {
    echo "Command failed!"
    exit 1
}
```

```rust
// RSB
// RSB fails fast by default
run_cmd("command_that_might_fail");  // Exits on failure

// Or explicit handling
let output = std::process::Command::new("risky_command")
    .output()
    .unwrap_or_else(|_| {
        eprintln!("Command failed!");
        std::process::exit(1);
    });
```

### Function Definitions

```bash
# Bash
check_requirements() {
    local missing=""
    for cmd in git docker; do
        if ! command -v "$cmd" >/dev/null; then
            missing="$missing $cmd"
        fi
    done
    [ -n "$missing" ] && {
        echo "Missing required commands:$missing"
        exit 1
    }
}
```

```rust
// RSB
fn check_requirements(_args: Args) {
    let required = ["git", "docker"];
    let mut missing = Vec::new();
    
    for cmd in required {
        let check = std::process::Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
            
        if !check {
            missing.push(cmd);
        }
    }
    
    if !missing.is_empty() {
        eprintln!("Missing required commands: {}", missing.join(" "));
        std::process::exit(1);
    }
}
```

## Conclusion

RSB is about choosing the right tool for the job. When you need a simple, maintainable CLI tool that's more powerful than bash but doesn't require the full complexity of idiomatic Rust, RSB provides a middle ground that emphasizes developer productivity and code clarity over theoretical purity.

Remember: RSB is a pattern, not a religion. Use it where it fits, and don't be afraid to mix in traditional Rust patterns when they make sense.