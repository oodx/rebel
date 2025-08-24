# RSB (Rebel String-Based) Rust

**RSB is a Rust library for writing simple, robust, and portable shell-like scripts.**

It's designed for the "too big for bash, too small for full Rust" sweet spot — tools and scripts that need more structure and safety than shell scripting can offer, but without the ceremonial complexity of idiomatic, production-grade Rust applications.

## The REBEL Philosophy

RSB is built on the **REBEL (Rust Bends to Ease Life)** philosophy, which prioritizes practitioner productivity over academic purity.

- **Accessibility Over Purity:** Familiar, bash-like patterns are chosen over complex Rust idioms to lower the cognitive load. If you know shell scripting, you should feel at home.
- **Pragmatic Safety:** Instead of relying solely on Rust's type system to prevent errors at compile-time, RSB provides a rich set of runtime validation functions and clear error messages.
- **Fail Fast, Fail Clear:** When something goes wrong, RSB exits with a helpful, colored message. No more wrestling with nested `Result` types for simple scripts.
- **String-First Design:** Strings are the universal interface of the shell. RSB embraces this, providing powerful tools to work with strings, rather than forcing you to convert everything into complex types.

RSB is for builders, for automators, for practitioners who need to "get stuff done."

## Core Architecture

RSB's design is heavily inspired by the mature **BashFX architecture**, incorporating battle-tested patterns for creating maintainable scripts:

- **XDG+ Compliance:** RSB tools are self-contained within the `~/.local` directory structure, keeping your home directory clean.
- **Function Ordinality:** A strict hierarchy for functions (High-Order, Mid-Level, Low-Level) ensures a clear separation of concerns and a predictable call stack.
- **Sentinel-Based Operations:** A system for making safe, reversible changes to files (e.g., for installers).
- **"Thisness" Pattern:** A context system that allows for the creation of generic, reusable library functions.

## Features

- **Bash-like Syntax:** Write scripts that feel like shell scripts, but with the power and safety of Rust.
- **Rich Macro DSL:** A powerful set of macros (`echo!`, `cmd!`, `cat!`, `test!`, `validate!`, `param!`, etc.) forms the core of the scripting experience.
- **Fluent Stream Processing:** Chain commands together to process text and data, just like Unix pipes.
- **Integrated Argument Parsing:** A simple yet powerful argument parser built-in.
- **Config File Loading:** Easily load `.env` or `.conf` style configuration files.
- **Colorized Output:** Built-in, configurable, and beautiful terminal output with colors and glyphs.
- **Job Control:** Run and manage background tasks with timeout support.
- **Event System:** A flexible `trap!` system for handling OS signals, script exit, and command errors.
- **Advanced String & Math Utilities**: Built-in random data generation and floating-point math.

## Getting Started

### 1. Add RSB to your project

```toml
# Cargo.toml
[dependencies]
rsb = { path = "path/to/rsb/crate" } # Or from crates.io when published
```

### 2. Write your first script

Create a new Rust binary project and add the following to your `main.rs`:

```rust
// main.rs
use rsb::prelude::*;

fn main() {
    // The bootstrap! macro handles collecting args, loading the environment,
    // and setting up the context all in one go.
    let args = bootstrap!();

    // Dispatch commands to their handler functions.
    dispatch!(&args, {
        "hello" => say_hello,
        "process" => process_files
    });
}

// A simple command handler
fn say_hello(args: Args) -> i32 {
    let name = args.get_or(1, "World");
    info!("Preparing to greet...");
    echo!("Hello, {}!", name);
    okay!("Greeting successful.");
    0
}

// A more complex handler showcasing stream processing
fn process_files(_args: Args) -> i32 {
    write_file("data.txt", "line 1\nline 2\nERROR: bad line\nline 4");
    require_file!("data.txt");
    info!("Processing data.txt...");

    let error_count = cat!("data.txt")
        .grep("ERROR")
        .tee("errors.log")
        .count();

    if error_count > 0 {
        error!("Found {} errors. See errors.log for details.", error_count);
        return 1;
    }

    okay!("No errors found.");
    0
}
```

### 3. Run it!

```sh
$ cargo run -- hello RSB
# ℹ Preparing to greet...
# Hello, RSB!
# ✓ Greeting successful.
```

## API Reference


### Core & Bootstrap

- **`bootstrap!() -> Vec<String>`**: Initializes the RSB environment (loads env vars, sets up paths) and returns the command-line arguments. The one-stop-shop for starting your script.
- **`args!() -> Vec<String>`**: A standalone macro to just get the command-line arguments.
- **`dispatch!(&args, { ... })`**: The main command router. Takes the arguments and a block mapping command strings to handler functions.
- **`pre_dispatch!(&args, { ... })`**: A secondary dispatcher for "bootstrap" commands (like `install` or `init`) that should run before config files are loaded.

### Logging & Output

- **`info!(...)`**: For general informational messages.
- **`okay!(...)`**: For success messages.
- **`warn!(...)`**: For warnings.
- **`error!(...)`**: For non-fatal errors.
- **`fatal!(...)`**: For fatal errors (red 💀).
- **`debug!(...)`**: For debug messages (grey 🔍).
- **`trace!(...)`**: For trace-level messages (magenta 👁).
- **`echo!(...)`**: Prints to `stdout`. Use this for output that needs to be piped or captured.
- **`printf!(...)`**: Like `echo!` but without a trailing newline.
- **`line!('-', 20)`**: Creates a string by repeating a character.
- **`clear!()`**: Clears the terminal screen.

### Variable & Config Management

- **`set_var(key, value)` / `get_var(key)`**: Get or set variables in the global context.
- **`param!(...)`**: A powerful macro for bash-style parameter expansion (e.g., `param!("VAR", default: "val")`, `param!("VAR", suffix: ".txt")`).
- **`src!(path, ...)` / `load_config!(path, ...)`**: Loads variables from one or more configuration files.
- **`export!(path)`**: Saves all context variables to a file in `export` format.
- **`meta_key!(path, key)`**: Extracts a single metadata value from a file's header comments.
- **`meta_keys!(path, into: "META")`**: Parses `# key: value` comments from a file and loads them into an associative array named `META`.

### Array & Dictionary Utilities
- **`set_array(name, &["a", "b"])`**: Creates an array variable.
- **`get_array(name) -> Vec<String>`**: Retrieves an array.
- **`array_push(name, item)`**: Appends an item to an array.
- **`for_in!(item in "ARRAY_NAME" => { ... })`**: Iterates over an RSB array.
- **`dict!(path)`**: Reads a whitespace-delimited file into a `Vec<String>`.
- **`gen_dict!(type, n, into: array_name)`**: Generates an array of `n` random "words" of a given `type` (`alnum`, `hex`, etc.).
- **`rand_dict!(array_name)`**: Returns a single random word from an array.

### Stream Processing
- **Sources**: `cat!(path)`, `cmd!(command)`, `pipe!(string)`, `stream!(array: &vec)`.
- **Methods**: `.grep()`, `.sed()`, `.cut()`, `.sort()`, `.unique()`, `.tee(path)`, `.to_file(path)`, `.each(|line| ...)`
- **`sed_block!(start, end, sed_expr)`**: Applies a `sed`-style substitution to a block of text between two patterns.
- **`cap_stream!(stream)` / `subst!(stream)`**: Captures a stream's output to a temporary file and returns the path. Useful for commands that require file paths instead of stdin (e.g., `diff`).

### Conditional Logic
- **`validate!(condition, message)`**: Exits with an error if the condition is false.
- **`require_file!(path)`**: Exits if the file does not exist.
- **`test!(...)`**: A comprehensive macro for bash-style tests (e.g., `test!(-f "file")`, `test!(var -gt 10)`).
- **`case!(value, { ... })`**: A shell-style `case` statement with regex pattern matching.

### System, Time & Math
- **`sleep!(1)` / `sleep!(ms: 100)`**: Pauses execution.
- **`date!(iso)` / `date!(epoch)` / `date!("%Y-%m-%d")`**: Gets the current time in various formats.
- **`benchmark!({ ... })`**: Measures the execution time of a code block.
- **`math!("VAR = (A + B) * 2")`**: Evaluates a mathematical expression (with float support) and assigns the result to a variable.
- **`tmp!()` / `tmp!(pid)`**: Generates a temporary file path.

### Job Control
- **`job!(background: "...")`**: Runs a command in the background and returns a job ID.
- **`job!(wait: id)`**: Waits for a background job to complete.
- **`job!(timeout: 10, wait: id)`**: Waits for a background job with a timeout in seconds.
- **`job!(list)`**: Lists running background jobs.

### Event Handling
- **`trap!(|| ..., on: "SIGINT")`**: Traps OS signals.
- **`trap!(|| ..., on: "EXIT")`**: Executes a handler when the script exits.
- **`trap!(|data| ..., on: "COMMAND_ERROR")`**: Executes a handler when `cmd!` or `shell!` fails. The `data` argument contains `source`, `command`, `status`, and `stderr`.

### Random Data Generation
- **`rand_alnum!(n)`**: Generates `n` random alphanumeric characters.
- **`rand_alpha!(n)`**: Generates `n` random alphabetic characters.
- **`rand_hex!(n)`**: Generates `n` random hex characters.
- **`rand_string!(n)`**: Generates `n` random printable characters.
- **`rand_uuid!()`**: Generates a v4 UUID.

Welcome to a more rebellious, productive way of writing scripts in Rust.
