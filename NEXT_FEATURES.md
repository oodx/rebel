# RSB - Next Features Roadmap

This document tracks the next set of features planned for the RSB library.

## Core Feature Set

### 1. Advanced Bash Parity Features
- **`math!` Macro**: A powerful, floating-point aware macro for shell-style arithmetic.
  - `math!("VAR = (OTHER_VAR * 1.05) + 2")`
  - `math!("COUNTER += 1")`
  - Should support: `+`, `-`, `*`, `/`, `%`, `**` (power), and shorthand assignments.
- **`cap_stream!` / `subst!` Macro**: A macro to support process substitution by capturing a stream's output to a temporary file and returning the path.
  - `let path = cap_stream!(cat!("file.txt").sort());`
  - `cmd!("diff {} /some/other/file.txt", path);`
  - Should use `$XDG_TMP` and have automatic cleanup on script exit.
- **`trap on ERR`**: Enhance `cmd!` and `shell!` to emit a `COMMAND_ERROR` event on non-zero exit codes, allowing for robust, script-wide error handling.
- **Robust `cp -r` Fallback**: Provide a more robust native Rust implementation for recursive copying to improve portability on systems without a standard `cp` command.

### 2. Foundational Utilities
- **`tmp!` Macro**: A macro to generate temporary file paths in a configurable temporary directory (`$XDG_TMP`).
  - `tmp!()` or `tmp!(random)`
  - `tmp!(pid)`
  - `tmp!(timestamp)`
- **Random Data Macros**: A suite of macros for generating random data.
  - `rand_alnum!(n)`
  - `rand_alpha!(n)`
  - `rand_hex!(n)`
  - `rand_string!(n)` (printable, non-whitespace)
  - `rand_uuid!`
- **Dictionary Macros**: Utilities for working with word lists.
  - `dict!(<filepath>)`: Loads a newline or space-delimited file into an RSB array.
  - `rand_dict!(<array_name>)`: Selects a random word from an RSB array.
  - `rand_dict!(<array_name>, n, <delim>)`: Creates a delimited string of `n` random words.
  - `gen_dict!(<type>, n)`: Generates an array of `n` random words of a given type (e.g., `alnum`, `hex`).

### 3. Quality of Life
- **`stderr!` Macro**: A macro for formatting strings with color codes without printing them to stderr.
  - `let my_str = stderr!("{red}Error!{reset}");`
