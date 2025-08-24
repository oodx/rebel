use crate::context::get_var;
use std::io::{self, BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::fs::File;

// The core struct for handling data streams
pub struct Stream {
    lines: Box<dyn Iterator<Item = String>>,
}

impl Stream {
    // --- Constructors ---
    pub fn from_string(s: &str) -> Self {
        let lines: Vec<String> = s.lines().map(|l| l.to_string()).collect();
        Stream { lines: Box::new(lines.into_iter()) }
    }
    pub fn from_delimited_string(s: &str, delimiter: &str) -> Self {
        let lines: Vec<String> = s.split(delimiter).map(|l| l.to_string()).collect();
        Stream { lines: Box::new(lines.into_iter()) }
    }
    pub fn from_vec(vec: Vec<String>) -> Self {
        Stream { lines: Box::new(vec.into_iter()) }
    }
    pub fn from_var(var_name: &str) -> Self {
        Self::from_string(&get_var(var_name))
    }
    pub fn from_file(path: &str) -> Self {
        let file = File::open(path).expect("Failed to open file");
        let reader = BufReader::new(file);
        let lines = reader.lines().map(|l| l.unwrap_or_default());
        Stream { lines: Box::new(lines) }
    }
    pub fn from_files(paths: &[&str]) -> Self {
        let mut combined_lines = Vec::new();
        for path in paths {
            if let Ok(file) = File::open(path) {
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    combined_lines.push(line.unwrap_or_default());
                }
            }
        }
        Stream { lines: Box::new(combined_lines.into_iter()) }
    }
    pub fn from_cmd(command: &str) -> Self {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "sh".to_string());
        let child = Command::new(shell)
            .arg("-c")
            .arg(command)
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to execute command");

        let reader = BufReader::new(child.stdout.expect("Failed to get stdout"));
        let lines = reader.lines().map(|l| l.unwrap_or_default());
        Stream { lines: Box::new(lines) }
    }

    // --- Chainable Methods ---
    pub fn map(self, func: impl Fn(String) -> String + 'static) -> Self {
        Stream { lines: Box::new(self.lines.map(func)) }
    }
    pub fn filter(self, func: impl Fn(&String) -> bool + 'static) -> Self {
        Stream { lines: Box::new(self.lines.filter(func)) }
    }
    pub fn grep(self, pattern: &str) -> Self {
        let re = regex::Regex::new(pattern).unwrap();
        self.filter(move |line| re.is_match(line))
    }
    pub fn sed(self, search: &str, replace: &str) -> Self {
        let re = regex::Regex::new(search).unwrap();
        let replace_owned = replace.to_string();
        self.map(move |line| re.replace_all(&line, &replace_owned).to_string())
    }
    pub fn head(self, count: usize) -> Self {
        Stream { lines: Box::new(self.lines.take(count)) }
    }
    pub fn tail(self, count: usize) -> Self {
        let lines: Vec<_> = self.lines.collect();
        let start = lines.len().saturating_sub(count);
        Stream { lines: Box::new(lines.into_iter().skip(start)) }
    }
    pub fn sort(self) -> Self {
        let mut lines: Vec<_> = self.lines.collect();
        lines.sort();
        Stream { lines: Box::new(lines.into_iter()) }
    }
    pub fn uniq(self) -> Self {
        let mut lines: Vec<_> = self.lines.collect();
        lines.dedup();
        Stream { lines: Box::new(lines.into_iter()) }
    }

    // --- Terminal Methods ---
    pub fn to_string(self) -> String {
        self.lines.collect::<Vec<String>>().join("\n")
    }
    pub fn to_vec(self) -> Vec<String> {
        self.lines.collect()
    }
    pub fn to_var(self, var_name: &str) {
        crate::context::set_var(var_name, &self.to_string());
    }
    pub fn to_file(self, path: &str) -> io::Result<()> {
        let mut file = File::create(path)?;
        for line in self.lines {
            writeln!(file, "{}", line)?;
        }
        Ok(())
    }
    pub fn print(self) {
        for line in self.lines {
            println!("{}", line);
        }
    }
}
