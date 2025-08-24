// src/prelude.rs

//! The RSB prelude.
//!
//! This module re-exports all the common traits, functions, and macros
//! for easy importing into user code via `use rsb::prelude::*;`.

// Re-export all public structs and functions.
pub use crate::args::Args;
pub use crate::context::{
    expand_vars, export_vars, get_var, has_var, load_config_file, parse_config_content,
    rsb_bootstrap, save_config_file, set_var, unset_var,
};
pub use crate::fs::*;
pub use crate::os::*;
pub use crate::streams::Stream;
pub use crate::utils::*;

// Re-export all macros.
pub use crate::{
    args, bootstrap, case, cat, cmd, date, debug, dispatch, echo, error, event, export, fatal,
    file_in, get_env, info, job, load_config, meta_key, meta_keys, okay, param, path_canon,
    path_split, pipe, pre_dispatch, printf, require_command, require_dir, require_file,
    require_var, shell, src, stream, test, trace, trap, validate, warn,
};
