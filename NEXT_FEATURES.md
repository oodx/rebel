# RSB - Next Features & Ideas

This document tracks potential features and improvements for future versions of the RSB library, based on ideas and feedback gathered during development.

## High Priority

### 1. Rich Date/Time Utilities
- **Description:** A full suite of `chrono`-based time and date utilities.
- **Features:**
  - `time_diff(start, end)`: Calculate the difference between two date/time strings.
  - `time_until(date)`: Produce a human-readable string like "in 2 hours".
  - `human_date(date)`: Format dates nicely (e.g., "3 days ago").
  - `benchmark! { ... }`: A macro to measure the execution time of a block of code.
  - `job!(timeout: 10s, ...)`: Add timeout support to the job control system.
  - Timezone support for date conversions.

### 2. Advanced `sed` with Block Operations
- **Description:** Enhance the `sed` stream operation to work on blocks of text delimited by patterns or sentinels, not just line-by-line. This would allow for complex, in-place editing of structured files or code.
- **Example:** `stream.sed_block("/start_pattern/", "/end_pattern/", "s/old/new/g")`

### 3. Configurable Stderr
- **Description:** Allow full user configuration of `stderr` output colors and glyphs via the `RSB_COLORS` environment variable.
- **Format:** `RSB_COLORS="info:[color,glyph],error:[color],fatal:[glyph]"`
- **Requires:** Refactoring the `COLORS` and `GLYPHS` statics to be fully mutable and parsing this variable at bootstrap.

### 4. Robust Job Control
- **Description:** The `job!(wait: ...)` implementation is currently a placeholder. A fully robust, thread-safe implementation for waiting on and retrieving results from background jobs is needed.
- **Possible Solution:** Investigate using channels (`std::sync::mpsc`) or a different concurrency primitive to safely manage `JoinHandle`s.

## Medium Priority

### 5. More Powerful String Utilities
- **Description:** Add more ergonomic string manipulation macros.
- **Features:**
    - `str_explode!(string, on: delim, into: arr_name)`: Split a string into an RSB array.
    - `str_in!(needle in haystack)`: A clean, readable macro for substring checks.
    - `str_trim!(var)`: Macro to trim whitespace from a variable.
    - `str_len!(var)`: Macro to get the length of a variable.

### 6. Stream from Array / Delimited String
- **Description:** Add constructors to `Stream` to easily create a stream from an existing `Vec<String>` or a delimited string.
- **Example:** `Stream::from_vec(&my_vec)`, `stream!(from_array: &my_vec)`

### 7. Glob Support in Parameter Expansion
- **Description:** The `param!` macro's prefix/suffix removal (`#`, `##`, `%`, `%%`) currently uses simple string matching. It should support shell-style glob patterns.
- **Example:** `param!("FILENAME", suffix: "*.log")`

### 8. `rsb_stderr!` Macro
- **Description:** A dedicated macro for color/glyph parsing, allowing users to easily format their own strings without necessarily printing to `stderr`.
- **Example:** `let formatted = rsb_stderr!("{red}My custom error{reset}");`

## Low Priority / Ideas

### 9. Native Windows Support
- **Description:** Currently, command execution relies on a `sh`-compatible shell. Add support for native Windows `cmd.exe` or `PowerShell` to improve portability. This would likely involve conditional compilation (`#[cfg(windows)]`). The `libc` dependency for signal handling would also need a Windows equivalent (`winapi`).

### 10. Official Testing Framework / Patterns
- **Description:** Document official patterns and provide helper functions for testing RSB scripts, especially for mocking file system or command execution.

### 11. Robust `cp -r` Fallback
- **Description:** The current `cp_r` fallback is very basic. A more robust, native Rust implementation could be provided for systems that don't have a `cp` command.
