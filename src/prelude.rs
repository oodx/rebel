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
pub use crate::time::*;
pub use crate::utils::*;
pub use crate::random::*;
pub use crate::math::*;

// Re-export all macros.
pub use crate::{
    args, backup, benchmark, bootstrap, case, cap_stream, cat, chmod, clear, cmd, confirm, date,
    debug, dict, dispatch, echo, error, event, export, fatal, file_in, for_in, get_env, gen_dict,
    info, job, load_config, math, meta_key, meta_keys, okay, param, path_canon, path_split, pipe,
    pre_dispatch, printf, prompt, rand_alnum, rand_alpha, rand_dict, rand_hex, rand_range,
    rand_string, rand_uuid, require_command, require_dir, require_file, require_var, shell, sleep,
    src, stream, str_explode, str_in, str_len, str_line, str_trim, subst, test, trace, trap,
    validate, warn,
};
