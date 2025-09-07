// RSB XCls - Closure-compatible extensions to RSB
//
// This module provides closure-supporting versions of RSB functions
// that enable complex transformations beyond simple string replacement.

pub mod xsed;

// Re-export main items
pub use xsed::{xsed, XSed, ToXSed};