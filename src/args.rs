use crate::context::expand_vars;
use std::collections::HashSet;

/// A struct for parsing command-line arguments in a bash-like manner.
///
/// It handles flags, options with values, key-value pairs, and arrays,
/// while keeping track of processed arguments to allow for easy access
/// to remaining positional arguments.
#[derive(Debug, Clone)]
pub struct Args {
    args: Vec<String>,
    processed: HashSet<usize>,
}

impl Args {
    /// Creates a new `Args` instance from a slice of strings.
    pub fn new(args: &[String]) -> Self {
        Args {
            args: args.to_vec(),
            processed: HashSet::new(),
        }
    }

    /// Gets the n-th unprocessed positional argument (1-indexed).
    /// Returns an empty string if the argument doesn't exist.
    pub fn get(&self, n: usize) -> String {
        if n == 0 {
            // In bash, $0 is the script name. In RSB, this is available via
            // `get_var("SCRIPT_NAME")`. Returning empty string for `get(0)`
            // avoids confusion.
            return "".to_string();
        }
        self.remaining().get(n - 1).cloned().unwrap_or_default()
    }

    /// Gets the n-th unprocessed positional argument (1-indexed), or a default value.
    pub fn get_or(&self, n: usize, default: &str) -> String {
        let val = self.get(n);
        if val.is_empty() {
            default.to_string()
        } else {
            val
        }
    }

    /// Checks if a flag exists in the arguments without consuming it.
    pub fn has(&self, flag: &str) -> bool {
        self.args.iter().any(|arg| arg == flag)
    }

    /// Checks if a flag exists and marks it as processed.
    pub fn has_pop(&mut self, flag: &str) -> bool {
        if let Some(pos) = self.args.iter().position(|arg| arg == flag) {
            self.processed.insert(pos);
            true
        } else {
            false
        }
    }

    /// Gets the value of a flag (e.g., `--file path.txt` or `--file=path.txt`)
    /// and marks both the flag and its value as processed.
    pub fn has_val(&mut self, flag: &str) -> Option<String> {
        // Check for --flag=value format
        for (i, arg) in self.args.iter().enumerate() {
            if let Some(value) = arg.strip_prefix(&format!("{}=", flag)) {
                if !self.processed.contains(&i) {
                    self.processed.insert(i);
                    return Some(value.to_string());
                }
            }
        }

        // Check for --flag value format
        if let Some(pos) = self.args.iter().position(|arg| arg == flag) {
            if !self.processed.contains(&pos) && pos + 1 < self.args.len() {
                self.processed.insert(pos);
                self.processed.insert(pos + 1);
                return Some(self.args[pos + 1].clone());
            }
        }

        None
    }

    /// Gets the value of a key-value pair argument (e.g., `output=/tmp` or `output:/tmp`)
    /// and marks it as processed.
    pub fn get_kv(&mut self, key: &str) -> Option<String> {
        for (i, arg) in self.args.iter().enumerate() {
            if self.processed.contains(&i) {
                continue;
            }
            if let Some(value) = arg.strip_prefix(&format!("{}=", key)) {
                self.processed.insert(i);
                return Some(value.to_string());
            }
            if let Some(value) = arg.strip_prefix(&format!("{}:", key)) {
                self.processed.insert(i);
                return Some(value.to_string());
            }
        }
        None
    }

    /// Gets an array from a key-value pair (e.g., `features=a,b,c`)
    /// and marks it as processed.
    pub fn get_array(&mut self, key: &str) -> Option<Vec<String>> {
        if let Some(value) = self.get_kv(key) {
            return Some(value.split(',').map(|s| s.trim().to_string()).collect());
        }
        None
    }

    /// Returns a vector of all unprocessed arguments.
    pub fn remaining(&self) -> Vec<String> {
        self.args
            .iter()
            .enumerate()
            .filter(|(i, _)| !self.processed.contains(i))
            .map(|(_, arg)| arg.clone())
            .collect()
    }

    /// Returns a slice of all original arguments.
    pub fn all(&self) -> &[String] {
        &self.args
    }

    /// Joins all unprocessed arguments with a separator.
    pub fn join(&self, sep: &str) -> String {
        self.remaining().join(sep)
    }

    /// Returns the number of unprocessed arguments.
    pub fn len(&self) -> usize {
        self.remaining().len()
    }

    /// Expands a template string, replacing placeholders with arguments and context variables.
    ///
    /// It replaces `$1`, `$2`, etc., with positional arguments, `$@` with all
    /// arguments, `$#` with the argument count, and then expands all
    /// context variables (e.g., `$HOME`).
    pub fn expand(&self, template: &str) -> String {
        let mut result = template.to_string();
        let remaining = self.remaining();

        // Replace positional args $1, $2, etc.
        for (i, arg) in remaining.iter().enumerate() {
            let placeholder = format!("${}", i + 1);
            result = result.replace(&placeholder, arg);
        }

        // Replace $@ and $#
        result = result.replace("$@", &self.join(" "));
        result = result.replace("$#", &self.len().to_string());

        // Expand context variables
        expand_vars(&result)
    }
}
