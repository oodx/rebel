// RSB Streamable - Unix-style function pipelines for Rust
// (Based on working XStream implementation)

pub mod traits;
pub mod functions;

// Re-export commonly used items
pub use traits::{Streamable, StreamApply};

// Re-export basic streamable structs
pub use functions::{
    Replace, UpperCase, LowerCase, Trim, Reverse,
    Base64Encode, Base64Decode, UrlEncode, UrlDecode,
    replace_fn, uppercase_fn, lowercase_fn,
};

// Re-export the streamable! macro
pub use crate::streamable;