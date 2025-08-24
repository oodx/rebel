// src/streams.rs

use crate::context::{expand_vars, get_var, set_var};
use crate::fs::{append_file, read_file, write_file};
use crate::os::run_cmd;
use std::collections::HashSet;
use std::io::Write;

/// A struct for building and executing bash-like data processing pipelines.
#[derive(Debug, Clone)]
pub struct Stream {
    lines: Vec<String>,
}

impl Stream {
    // --- Constructors ---

    /// Creates a new, empty stream.
    pub fn new() -> Self {
        Stream { lines: Vec::new() }
    }

    /// Creates a stream from a string.
    pub fn from_string(content: &str) -> Self {
        Stream {
            lines: content.lines().map(|s| s.to_string()).collect(),
        }
    }

    /// Creates a stream from the contents of a file.
    pub fn from_file(path: &str) -> Self {
        let content = read_file(&expand_vars(path));
        Self::from_string(&content)
    }

    /// Creates a stream from the contents of multiple files.
    pub fn from_files(paths: &[&str]) -> Self {
        let mut content = String::new();
        for path in paths {
            let file_content = read_file(&expand_vars(path));
            if !content.is_empty() {
                content.push('\n');
            }
            content.push_str(&file_content);
        }
        Self::from_string(&content)
    }

    /// Creates a stream from the stdout of a shell command.
    pub fn from_cmd(cmd: &str) -> Self {
        let output = run_cmd(&expand_vars(cmd));
        Self::from_string(&output)
    }

    /// Creates a stream from an RSB context variable.
    pub fn from_var(var_name: &str) -> Self {
        let content = get_var(var_name);
        Self::from_string(&content)
    }

    /// Creates a stream from a vector of strings.
    pub fn from_vec(lines: &[String]) -> Self {
        Stream {
            lines: lines.to_vec(),
        }
    }

    /// Creates a stream from a single string by splitting it with a delimiter.
    pub fn from_delimited_string(content: &str, delimiter: &str) -> Self {
        Stream {
            lines: content.split(delimiter).map(|s| s.to_string()).collect(),
        }
    }


    // --- Chainable Operations ---

    /// Filters lines in the stream, keeping only those that contain the pattern.
    pub fn grep(mut self, pattern: &str) -> Self {
        self.lines.retain(|line| line.contains(pattern));
        self
    }

    /// Replaces all occurrences of a pattern in the stream.
    pub fn sed(mut self, from: &str, to: &str) -> Self {
        self.lines = self.lines.iter().map(|line| line.replace(from, to)).collect();
        self
    }

    /// Extracts a specific field from each line, based on a delimiter. (1-indexed)
    pub fn cut(mut self, field: usize, delimiter: &str) -> Self {
        self.lines = self
            .lines
            .iter()
            .filter_map(|line| line.split(delimiter).nth(field.saturating_sub(1)))
            .map(|s| s.to_string())
            .collect();
        self
    }

    /// Takes the first `n` lines of the stream.
    pub fn head(mut self, n: usize) -> Self {
        self.lines.truncate(n);
        self
    }

    /// Takes the last `n` lines of the stream.
    pub fn tail(mut self, n: usize) -> Self {
        let len = self.lines.len();
        if len > n {
            self.lines = self.lines.into_iter().skip(len - n).collect();
        }
        self
    }

    /// Sorts the lines in the stream alphabetically.
    pub fn sort(mut self) -> Self {
        self.lines.sort();
        self
    }

    /// Removes duplicate consecutive lines from the stream.
    pub fn uniq(mut self) -> Self {
        self.lines.dedup();
        self
    }

    /// Removes all duplicate lines from the stream, regardless of order.
    pub fn unique(mut self) -> Self {
        let mut seen = HashSet::new();
        self.lines.retain(|line| seen.insert(line.clone()));
        self
    }

    /// Applies a custom filter function to each line.
    pub fn filter<F>(mut self, predicate: F) -> Self
    where
        F: Fn(&str) -> bool,
    {
        self.lines.retain(|line| predicate(line));
        self
    }

    /// Applies a custom mapping function to each line.
    pub fn map<F>(mut self, mapper: F) -> Self
    where
        F: Fn(&str) -> String,
    {
        self.lines = self.lines.iter().map(|line| mapper(line)).collect();
        self
    }

    /// Replaces a block of text between two patterns.
    pub fn sed_block(mut self, start_pattern: &str, end_pattern: &str, replacement: &str) -> Self {
        let mut result_lines = Vec::new();
        let mut buffer = Vec::new();
        let mut in_block = false;

        for line in self.lines {
            if !in_block && line.contains(start_pattern) {
                in_block = true;
                // The line that starts the block is part of the block
                buffer.push(line);
            } else if in_block && line.contains(end_pattern) {
                in_block = false;
                buffer.push(line);
                // Perform the replacement on the entire block
                let block_content = buffer.join("\n");
                result_lines.push(block_content.replace(start_pattern, replacement).replace(end_pattern, ""));
                buffer.clear();
            } else if in_block {
                buffer.push(line);
            } else {
                result_lines.push(line);
            }
        }
        // What if the end pattern is never found? Append the buffer.
        if !buffer.is_empty() {
            result_lines.extend(buffer);
        }

        self.lines = result_lines;
        self
    }


    /// Pipes the stream's content as stdin to another shell command.
    pub fn pipe_to_cmd(self, command: &str) -> Self {
        let input = self.to_string();
        let expanded_cmd = expand_vars(command);

        let mut child = std::process::Command::new("sh")
            .arg("-c")
            .arg(&expanded_cmd)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn command for pipe_to_cmd");

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(input.as_bytes()).expect("Failed to write to stdin");
        }

        let output = child.wait_with_output().expect("Failed to read stdout from piped command");
        Self::from_string(&String::from_utf8_lossy(&output.stdout))
    }


    // --- Sink (Consuming) Operations ---

    /// Returns the stream's contents as a single string.
    pub fn to_string(&self) -> String {
        self.lines.join("\n")
    }

    /// Consumes the stream and returns its contents as a vector of strings.
    pub fn to_vec(self) -> Vec<String> {
        self.lines
    }

    /// Writes the stream's contents to a file, then consumes the stream.
    pub fn to_file(self, path: &str) {
        write_file(&expand_vars(path), &self.to_string());
    }

    /// Writes the stream's contents to a variable, then returns the stream.
    pub fn to_var(self, var_name: &str) -> Self {
        set_var(var_name, &self.to_string());
        self
    }

    /// Appends the stream's contents to a file, then consumes the stream.
    pub fn append_to_file(self, path: &str) {
        append_file(&expand_vars(path), &self.to_string());
    }

    /// Writes the stream's contents to a file, but returns the stream for further processing.
    pub fn tee(&self, path: &str) -> Self {
        write_file(&expand_vars(path), &self.to_string());
        self.clone()
    }

    /// Executes an action for each line in the stream, then returns the stream.
    pub fn each<F>(self, action: F) -> Self
    where
        F: Fn(&str),
    {
        for line in &self.lines {
            action(line);
        }
        self
    }

    /// Consumes the stream and returns the number of lines.
    pub fn count(self) -> usize {
        self.lines.len()
    }

    /// Returns the first line of the stream, if any.
    pub fn first(&self) -> Option<&String> {
        self.lines.first()
    }

    /// Returns the last line of the stream, if any.
    pub fn last(&self) -> Option<&String> {
        self.lines.last()
    }
}

impl Default for Stream {
    fn default() -> Self {
        Self::new()
    }
}
