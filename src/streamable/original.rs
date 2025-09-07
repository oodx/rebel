// XStream Streamable - Prototype for generalized pipe-able functions
// This will eventually be pushed upstream to RSB!

use rsb::prelude::*;

/// Streamable trait - functions that take stdin + args and produce stdout
/// The unix pipe pattern generalized for any function
pub trait Streamable {
    type Args;
    
    /// Apply this streamable function to stdin with given args
    fn stream_apply(stdin: &str, args: Self::Args) -> String;
}

/// Macro to create streamable functions easily
#[macro_export]
macro_rules! streamable {
    // Simple case: fn_name(stdin, arg1, arg2) => { body }
    ($fn_name:ident($stdin:ident, $($arg:ident: $arg_type:ty),*) => $body:block) => {
        pub struct $fn_name;
        
        impl Streamable for $fn_name {
            type Args = ($($arg_type,)*);
            
            fn stream_apply(stdin: &str, args: Self::Args) -> String {
                let $stdin = stdin;
                #[allow(unused_variables)]
                let ($($arg,)*) = args;
                $body
            }
        }
    };
}

// === STREAMABLE FUNCTION IMPLEMENTATIONS ===

// Text transforms
streamable!(Replace(stdin, find: String, replace: String) => {
    stdin.replace(&find, &replace)
});

streamable!(UpperCase(stdin,) => {
    stdin.to_uppercase()
});

streamable!(LowerCase(stdin,) => {
    stdin.to_lowercase()
});

streamable!(Reverse(stdin,) => {
    stdin.chars().rev().collect()
});

streamable!(Length(stdin,) => {
    stdin.len().to_string()
});

// Unix-style functions
streamable!(Head(stdin, n: usize) => {
    stdin.lines().take(n).collect::<Vec<_>>().join("\n")
});

streamable!(Tail(stdin, n: usize) => {
    let lines: Vec<&str> = stdin.lines().collect();
    lines.iter().skip(lines.len().saturating_sub(n)).cloned().collect::<Vec<_>>().join("\n")
});

streamable!(Grep(stdin, pattern: String) => {
    stdin.lines()
        .filter(|line| line.contains(&pattern))
        .collect::<Vec<_>>()
        .join("\n")
});

streamable!(Sort(stdin,) => {
    let mut lines: Vec<&str> = stdin.lines().collect();
    lines.sort();
    lines.join("\n")
});

streamable!(Unique(stdin,) => {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    stdin.lines()
        .filter(|line| seen.insert(*line))
        .collect::<Vec<_>>()
        .join("\n")
});

streamable!(WordCount(stdin,) => {
    let lines = stdin.lines().count();
    let words = stdin.split_whitespace().count(); 
    let chars = stdin.chars().count();
    format!("{} {} {}", lines, words, chars)
});

// Token-specific functions
streamable!(TokenCount(stdin,) => {
    stdin.split(';').filter(|s| !s.trim().is_empty()).count().to_string()
});

streamable!(ExtractKeys(stdin,) => {
    stdin.split(';')
        .filter_map(|token| {
            token.trim().split('=').next().map(|s| s.trim())
        })
        .collect::<Vec<_>>()
        .join("\n")
});

streamable!(ExtractValues(stdin,) => {
    stdin.split(';')
        .filter_map(|token| {
            token.trim().split('=').nth(1).map(|s| s.trim_matches('"').trim_matches('\''))
        })
        .collect::<Vec<_>>()
        .join("\n")
});

streamable!(FilterTokens(stdin, key_contains: String) => {
    stdin.split(';')
        .filter(|token| token.contains(&key_contains))
        .collect::<Vec<_>>()
        .join("; ")
});

// Encoding streamables  
streamable!(Base64Encode(stdin,) => {
    use base64::{Engine as _, engine::general_purpose};
    general_purpose::STANDARD.encode(stdin.as_bytes())
});

streamable!(Base64Decode(stdin,) => {
    use base64::{Engine as _, engine::general_purpose};
    general_purpose::STANDARD
        .decode(stdin)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .unwrap_or_else(|| stdin.to_string())
});

streamable!(UrlEncode(stdin,) => {
    urlencoding::encode(stdin).to_string()
});

streamable!(UrlDecode(stdin,) => {
    urlencoding::decode(stdin)
        .map(|s| s.to_string())
        .unwrap_or_else(|_| stdin.to_string())
});

// RSB-style sed operations
streamable!(Sed(stdin, pattern: String, replacement: String) => {
    stream!(string: stdin)
        .sed(&pattern, &replacement)
        .to_string()
});

streamable!(SedLines(stdin, start: usize, end: usize) => {
    stream!(string: stdin)
        .sed_lines(start, end)
        .to_string()
});

// Advanced streamables
streamable!(Pipeline(stdin, commands: Vec<String>) => {
    let mut result = stdin.to_string();
    for cmd in commands {
        // This could parse and execute simple commands
        // For now, just return the result
        result = format!("# Executed: {}\n{}", cmd, result);
    }
    result
});

// JSON-like operations (if we had JSON support)
streamable!(JsonGet(stdin, path: String) => {
    // Would parse JSON and extract path
    format!("JSON[{}]: {}", path, stdin.lines().next().unwrap_or(""))
});

/// Helper trait to make any type streamable in a pipeline
pub trait StreamApply {
    fn stream_apply<S: Streamable>(self, streamable: S, args: S::Args) -> Self;
}

// We'll implement this for TokenStream next!