// --- Bootstrap & Args Macros ---
// Namespaced re-exports for selective imports
pub use crate::{bootstrap, args, get_env};
#[macro_export]
macro_rules! bootstrap {
    () => {{
        let args: Vec<String> = std::env::args().collect();
        $crate::get_env!();
        $crate::context::rsb_bootstrap(&args);
        // Register a trap to clean up temp files on exit.
        $crate::trap!(|_: &$crate::os::EventData| {
            $crate::fs::cleanup_temp_files();
        }, on: "EXIT");
        args
    }};
}

#[macro_export]
macro_rules! args {
    () => {
        std::env::args().collect::<Vec<String>>()
    };
}

#[macro_export]
macro_rules! get_env {
    () => {
        for (key, value) in std::env::vars() {
            $crate::context::set_var(&key, &value);
        }
    };
}
