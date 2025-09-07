// RSB Streamable - Unix-style function pipelines for Rust
// (Based on working XStream implementation)

pub mod traits;
pub mod functions;

// Re-export commonly used items
pub use traits::{Streamable, StreamApply};

// Re-export basic streamable structs
pub use functions::{
    // Basic text transforms
    Replace, UpperCase, LowerCase, Trim, Reverse,
    // Encoding transforms
    Base64Encode, Base64Decode, UrlEncode, UrlDecode,
    // Unix-style streamables
    Head, Tail, Grep, Sort, Unique, WordCount,
    // Token-specific streamables  
    TokenCount, ExtractKeys, ExtractValues, FilterTokens,
    // RSB integration streamables
    Sed, SedLines,
    // Function-style interfaces
    replace_fn, uppercase_fn, lowercase_fn,
};

// Re-export the streamable! macro
pub use crate::streamable;