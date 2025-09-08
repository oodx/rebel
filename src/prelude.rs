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
pub use crate::streamable::{
    Streamable, StreamApply,
    // Advanced streamables for Pattern 2 & 3 composability
    Replace, UpperCase, LowerCase, Trim, Reverse,
    Base64Encode, Base64Decode, UrlEncode, UrlDecode,
    Head, Tail, Grep, Sort, Unique, WordCount,
    Sed, SedLines,
};

// Re-export the streamable! macro
pub use crate::streamable;
pub use crate::streams::Stream;
pub use crate::time::*;
pub use crate::utils::*;
pub use crate::xcls::{xsed, XSed, ToXSed};
pub use crate::random::*;
pub use crate::math::*;

// Re-export external dependencies so users importing the prelude
// also get convenient access to third-party crates RSB depends on.
pub use crate::deps::*;

// Re-export all macros.
pub use crate::{
    args, backup, benchmark, bootstrap, case, cap_stream, cat, chmod, cmd, colored,
    current_dir, curl, date, debug, dict, dispatch, echo, error, event, export, fatal, file_in, for_in, get, get_env, gen_dict, home_dir, hostname, info, job, json_get, json_get_file,
    kill_pid, kill_process, load_config, lock, math, mock_cmd, okay, pack, param,
    pid_of, pipe, pre_dispatch, printf, process_exists,
    rand_alnum, rand_alpha, rand_dict, rand_hex, rand_string, rand_uuid,
    rand_range,
    readline, require_command, require_dir, require_file, require_var, sed_around, sed_around_file,
    sed_insert, sed_insert_file, sed_lines, sed_lines_file, sed_replace, sed_template,
    sed_template_file, run, shell, src, stderr, stream, str_explode, str_in, str_len,
    str_trim, str_line, subst, tar, tar_gz, test, tmp, to_number, trace, trap, unpack, unlock, user, validate, warn,
    with_lock, zip, sleep, path_canon, path_split, meta_keys,
};

// Re-export macro groups for selective imports via the prelude.
pub mod macros {
    pub use crate::macros::*;
}
