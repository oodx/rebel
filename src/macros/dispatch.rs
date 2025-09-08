// --- Dispatch Macros ---
// Namespaced re-exports for selective imports
pub use crate::{dispatch, pre_dispatch};
#[macro_export]
macro_rules! dispatch {
    ($args:expr, { $($cmd:literal => $handler:ident),* }) => {
        {
          let args_vec: &Vec<String> = $args;
          let command = args_vec.get(1).map(|s| s.as_str()).unwrap_or("help");
          // Safely forward all args after the command (if present)
          let start_idx = if args_vec.len() > 2 { 2 } else { args_vec.len() };
          let cmd_args = $crate::args::Args::new(&args_vec[start_idx..]);
          $( $crate::context::register_function($cmd, stringify!($handler)); )*
          
          match command {
              $($cmd => {
                  $crate::context::push_call($cmd, cmd_args.all());
                  let result = $handler(cmd_args);
                  $crate::context::pop_call();
                  std::process::exit(result);
              },)*
              "help" | "--help" | "-h" => { 
                  $crate::context::show_help(); 
                  std::process::exit(0);
              },
              "inspect" => { 
                  $crate::context::show_functions(); 
                  std::process::exit(0);
              },
              "stack" => { 
                  $crate::context::show_call_stack(); 
                  std::process::exit(0);
              },
              _ => { 
                  $crate::error!("Unknown command: {}", command); 
                  $crate::context::show_help(); 
                  std::process::exit(1);
              }
          }
        }
    };
}

#[macro_export]
macro_rules! pre_dispatch {
    ($args:expr, { $($cmd:literal => $handler:ident),* }) => {
        {
            let args_vec: &Vec<String> = $args;
            let command = args_vec.get(1).map(|s| s.as_str()).unwrap_or("");
            
            // Detect if running in test environment
            let is_test = std::env::var("CARGO_TEST").is_ok() || std::thread::current().name().map_or(false, |name| name.contains("test"));
            
            match command {
                $($cmd => {
                    // Safely forward all args after the command
                    let start_idx = if args_vec.len() > 2 { 2 } else { args_vec.len() };
                    let cmd_args = $crate::args::Args::new(&args_vec[start_idx..]);
                    $crate::context::push_call($cmd, cmd_args.all());
                    let result = $handler(cmd_args);
                    $crate::context::pop_call();
                    if is_test {
                        true
                    } else {
                        std::process::exit(result);
                    }
                },)*
                _ => { false }
            }
        }
    };
}
