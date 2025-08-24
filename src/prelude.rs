//! The RSB Prelude.
//!
//! The purpose of this module is to alleviate imports of many common rsb traits
//! by adding a glob import to the top of rsb programs:
//!
//! ```
//! # // The following line is not tested because it would require this crate to be
//! # // a dependency, which it is not.
//! # // use rsb::prelude::*;
//! ```

// --- MACROS (sorted alphabetically) ---
// These are all exported at the crate root.
pub use crate::{
    args, backup, benchmark, bootstrap, case, cat, chmod, clear, cmd, confirm, date, db_conn,
    db_exec, db_query, debug, dispatch, echo, error, event, export, fatal, file_in, for_in,
    get_env, http_server, info, job, jwt_sign, jwt_verify, load_config, meta_key, meta_keys,
    okay, param, password_hash, password_verify, path_canon, path_split, pipe, pre_dispatch,
    printf, prompt, rand_range, require_command, require_dir, require_file, require_var, route,
    shell, sleep, src, stream, str_explode, str_in, str_len, str_line, str_trim, test, trace,
    trap, validate, warn,
};

// --- CORE STRUCTS & FUNCTIONS ---
pub use crate::args::Args;
pub use crate::context::{
    get_var, set_var, unset_var, expand_vars, has_var,
    Context, CallFrame,
};
pub use crate::streams::Stream;
pub use crate::fs::{
    self as rsb_fs, // alias to avoid conflicts
    is_dir, is_entity, is_executable, is_file, is_link, is_nonempty_file, is_readable,
    is_writable, read_file, write_file, file_append, file_out,
};
pub use crate::os::{
    self as rsb_os, // alias to avoid conflicts
    is_command, os_arch, os_family, os_type, os_cpus, os_hostname, os_homedir, os_tmpdir,
    shell_exec,
};
pub use crate::utils::{
    self as rsb_utils, // alias to avoid conflicts
    array_push, confirm_action, get_array, prompt_user, set_array, str_equals, str_matches,
    str_replace, str_sub, str_lower, str_upper, num_eq, num_gt, num_lt, glyph_stderr,
};

// --- SAAS STRUCTS & FUNCTIONS ---
pub use crate::auth::{
    auth_jwt_create, auth_jwt_verify, auth_hash_password, auth_verify_password
};
pub use crate::db::{
    db_exec as db_execute, db_query as db_query_rows
};
pub use crate::web::{
    web_route, web_start, Request as WebRequest
};
