use crate::context::expand_vars;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct Args {
    args: Vec<String>,
    processed: HashSet<usize>,
}

impl Args {
    pub fn new(args: &[String]) -> Self {
        Args {
            args: args.to_vec(),
            processed: HashSet::new(),
        }
    }
    pub fn get(&self, n: usize) -> String {
        if n == 0 { return "".to_string(); }
        self.remaining().get(n - 1).cloned().unwrap_or_default()
    }
    pub fn get_or(&self, n: usize, default: &str) -> String {
        let val = self.get(n);
        if val.is_empty() { default.to_string() } else { val }
    }
    pub fn has(&self, flag: &str) -> bool {
        self.args.iter().any(|arg| arg == flag)
    }
    pub fn has_pop(&mut self, flag: &str) -> bool {
        if let Some(pos) = self.args.iter().position(|arg| arg == flag) {
            self.processed.insert(pos);
            true
        } else {
            false
        }
    }
    pub fn has_val(&mut self, flag: &str) -> Option<String> {
        if let Some(pos) = self.args.iter().position(|a| a.starts_with(&format!("{}=", flag))) {
            if !self.processed.contains(&pos) {
                self.processed.insert(pos);
                let arg = &self.args[pos];
                return arg.split('=').nth(1).map(|s| s.to_string());
            }
        }
        if let Some(pos) = self.args.iter().position(|arg| arg == flag) {
            if !self.processed.contains(&pos) && pos + 1 < self.args.len() {
                self.processed.insert(pos);
                self.processed.insert(pos + 1);
                return Some(self.args[pos + 1].clone());
            }
        }
        None
    }
    pub fn get_kv(&mut self, key: &str) -> Option<String> {
        for (i, arg) in self.args.iter().enumerate() {
            if self.processed.contains(&i) { continue; }
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
    pub fn get_array(&mut self, key: &str) -> Option<Vec<String>> {
        self.get_kv(key).map(|v| v.split(',').map(|s| s.trim().to_string()).collect())
    }
    pub fn remaining(&self) -> Vec<String> {
        self.args.iter().enumerate()
            .filter(|(i, _)| !self.processed.contains(i))
            .map(|(_, arg)| arg.clone())
            .collect()
    }
    pub fn all(&self) -> &[String] { &self.args }
    pub fn join(&self, sep: &str) -> String { self.remaining().join(sep) }
    pub fn len(&self) -> usize { self.remaining().len() }
    pub fn expand(&self, template: &str) -> String {
        let mut result = template.to_string();
        let remaining = self.remaining();
        for (i, arg) in remaining.iter().enumerate() {
            result = result.replace(&format!("${}", i + 1), arg);
        }
        result = result.replace("$@", &self.join(" "));
        result = result.replace("$#", &self.len().to_string());
        expand_vars(&result)
    }
}
