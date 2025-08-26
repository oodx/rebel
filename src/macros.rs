// src/macros.rs

// --- Bootstrap & Args Macros ---
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

// --- Dispatch Macros ---
#[macro_export]
macro_rules! dispatch {
    ($args:expr, { $($cmd:literal => $handler:ident),* }) => {
        {
            let args_vec: &Vec<String> = $args;
            let command = args_vec.get(1).map(|s| s.as_str()).unwrap_or("help");
            let cmd_args = $crate::args::Args::new(&args_vec[2..]);
            $( $crate::context::register_function($cmd, stringify!($handler)); )*
            match command {
                $($cmd => {
                    $crate::context::push_call($cmd, cmd_args.all());
                    let result = $handler(cmd_args);
                    $crate::context::pop_call();
                    std::process::exit(result);
                },)*
                "help" | "--help" | "-h" => { $crate::context::show_help(); std::process::exit(0); },
                "inspect" => { $crate::context::show_functions(); std::process::exit(0); },
                "stack" => { $crate::context::show_call_stack(); std::process::exit(0); },
                _ => { $crate::error!("Unknown command: {}", command); $crate::context::show_help(); std::process::exit(1); }
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
            match command {
                $($cmd => {
                    let cmd_args = $crate::args::Args::new(&args_vec[2..]);
                    $crate::context::push_call($cmd, cmd_args.all());
                    let result = $handler(cmd_args);
                    $crate::context::pop_call();
                    std::process::exit(result);
                },)*
                _ => { false }
            }
        }
    };
}

// --- Stream Macros ---
#[macro_export]
macro_rules! cat {
    ($path:expr) => { $crate::streams::Stream::from_file($path) };
    ($($path:expr),+) => { $crate::streams::Stream::from_files(&[$($path),+]) };
}
#[macro_export]
macro_rules! cmd { ($command:expr) => { $crate::streams::Stream::from_cmd($command) }; }
#[macro_export]
macro_rules! pipe { ($input:expr) => { $crate::streams::Stream::from_string(&$input.to_string()) }; }
#[macro_export]
macro_rules! stream {
    (var: $var:expr) => { $crate::streams::Stream::from_var($var) };
    (files: $($path:expr),+) => { $crate::streams::Stream::from_files(&[$($path),+]) };
    (cmd: $command:expr) => { $crate::streams::Stream::from_cmd($command) };
    (string: $content:expr) => { $crate::streams::Stream::from_string($content) };
    (array: $arr:expr) => { $crate::streams::Stream::from_vec($arr) };
    (delimited: $content:expr, on: $delim:expr) => { $crate::streams::Stream::from_delimited_string($content, $delim) };
}
#[macro_export]
macro_rules! run {
    ($($arg:tt)*) => {
        {
            let cmd_str = format!($($arg)*);
            match $crate::os::shell_exec(&cmd_str, false) {
                Ok(output) => output,
                Err(res) => {
                    $crate::event!(emit "COMMAND_ERROR", "source" => "run!", "command" => &cmd_str, "status" => &res.status.to_string(), "stderr" => &res.error);
                    $crate::fatal!("Shell command failed: {}", cmd_str);
                    std::process::exit(res.status);
                }
            }
        }
    };
    ($($arg:tt)*, silent) => {
        match $crate::os::shell_exec(&format!($($arg)*), true) {
            Ok(output) => output,
            Err(_) => String::new(),
        }
    };
}

#[macro_export]
macro_rules! shell {
    ($($arg:tt)*) => {
        {
            let cmd_str = format!($($arg)*);
            $crate::os::run_cmd_with_status(&cmd_str)
        }
    };
}

// --- I/O Macros ---
#[macro_export]
macro_rules! readline {
    () => {
        {
            let mut input = String::new();
            match std::io::stdin().read_line(&mut input) {
                Ok(_) => input.trim().to_string(),
                Err(_) => String::new(),
            }
        }
    };
    ($prompt:expr) => {
        {
            eprint!("{}", $prompt);
            let _ = std::io::stderr().flush();
            let mut input = String::new();
            match std::io::stdin().read_line(&mut input) {
                Ok(_) => input.trim().to_string(),
                Err(_) => String::new(),
            }
        }
    };
}

#[macro_export]
macro_rules! stderr {
    ($($arg:tt)*) => {
        {
            let msg = format!($($arg)*);
            eprintln!("{}", msg);
        }
    };
}


#[macro_export]
macro_rules! to_number {
    ($text:expr) => {
        {
            $text.trim().parse::<i32>().unwrap_or(0)
        }
    };
    ($text:expr, default: $default:expr) => {
        {
            $text.trim().parse::<i32>().unwrap_or($default)
        }
    };
}

// --- Job Control Macros ---
#[macro_export]
macro_rules! job {
    (background: $command:expr) => {{
        let mut counter = $crate::os::JOB_COUNTER.lock().unwrap();
        *counter += 1;
        let job_id = *counter;
        let cmd_string = $command.to_string();
        let (tx, rx) = std::sync::mpsc::channel();
        let cmd_string_for_thread = cmd_string.clone();

        let handle = std::thread::spawn(move || {
            let result = $crate::os::run_cmd_with_status(&cmd_string_for_thread);
            let _ = tx.send(result);
        });

        let job_handle = $crate::os::JobHandle {
            id: job_id,
            command: cmd_string,
            handle: Some(handle),
            rx: rx,
        };
        $crate::os::JOBS.lock().unwrap().insert(job_id, std::sync::Arc::new(std::sync::Mutex::new(job_handle)));
        $crate::info!("[{}] Started background job", job_id);
        job_id
    }};
    (wait: $job_id:expr) => {{
        $crate::info!("[{}] Waiting for job to complete...", $job_id);
        match $crate::os::wait_on_job($job_id, None) {
            Ok(result) => result.status,
            Err(e) => {
                $crate::error!("Failed to wait for job {}: {}", $job_id, e);
                -1
            }
        }
    }};
    (timeout: $timeout:expr, wait: $job_id:expr) => {{
        $crate::info!("[{}] Waiting for job to complete (timeout: {}s)...", $job_id, $timeout);
        let timeout_duration = std::time::Duration::from_secs($timeout);
        match $crate::os::wait_on_job($job_id, Some(timeout_duration)) {
            Ok(result) => result.status,
            Err(e) => {
                $crate::error!("Failed to wait for job {}: {}", $job_id, e);
                -1
            }
        }
    }};
    (list) => {{
        let jobs = $crate::os::JOBS.lock().unwrap();
        if jobs.is_empty() {
            $crate::info!("No running jobs.");
        } else {
            $crate::info!("Running jobs:");
            for (id, job_mutex) in jobs.iter() {
                let job = job_mutex.lock().unwrap();
                $crate::echo!("[{}] {}", id, job.command);
            }
        }
    }};
}

// --- Event Macros ---
#[macro_export]
macro_rules! event {
    (register $event:expr, $handler:expr) => {{
        let mut handlers = $crate::os::EVENT_HANDLERS.lock().unwrap();
        let event_handlers = handlers.entry($event.to_string()).or_insert_with(Vec::new);
        event_handlers.push(Box::new($handler));
    }};
    (emit $event:expr, $($key:expr => $value:expr),*) => {{
        let mut data = ::std::collections::HashMap::new();
        $( data.insert($key.to_string(), $value.to_string()); )*
        let event_data = $crate::os::EventData { event_type: $event.to_string(), data, };
        if let Some(handlers) = $crate::os::EVENT_HANDLERS.lock().unwrap().get($event) {
            for handler in handlers {
                handler(&event_data);
            }
        }
    }};
}
#[macro_export]
macro_rules! trap {
    ($handler:expr, on: $signal:expr) => {{
        let sig_name = $signal.to_uppercase();
        match sig_name.as_str() {
            "SIGINT" | "SIGTERM" | "EXIT" | "COMMAND_ERROR" => {
                $crate::os::install_signal_handlers();
                $crate::event!(register &sig_name, $handler);
            }
            _ => { $crate::event!(register &sig_name, $handler); }
        }
    }};
    (on_file_read $handler:expr) => { $crate::event!(register "file_read", $handler); };
    (on_pipe_complete $handler:expr) => { $crate::event!(register "pipe_complete", $handler); };
    (on_command_start $handler:expr) => { $crate::event!(register "command_start", $handler); };
}


// --- Loop Macros ---
/// A macro for shell-style `for in` loops over RSB arrays.

#[macro_export]
macro_rules! for_in {
    ($var:ident in $array_name:expr => $body:block) => {
        for item in $crate::utils::get_array($array_name) {
            $crate::context::set_var(stringify!($var), &item);
            $body
        }
    };
    ($index:ident, $var:ident in $array_name:expr => $body:block) => {
        for (i, item) in $crate::utils::get_array($array_name).iter().enumerate() {
            $crate::context::set_var(stringify!($index), &i.to_string());
            $crate::context::set_var(stringify!($var), item);
            $body
        }
    };
}


// --- Logic and Control Flow Macros ---
#[macro_export]
macro_rules! test {
    (-e $path:expr) => { $crate::fs::is_entity($path) };
    (-f $path:expr) => { $crate::fs::is_file($path) };
    (-d $path:expr) => { $crate::fs::is_dir($path) };
    (-L $path:expr) => { $crate::fs::is_link($path) };
    (-r $path:expr) => { $crate::fs::is_readable($path) };
    (-w $path:expr) => { $crate::fs::is_writable($path) };
    (-x $path:expr) => { $crate::fs::is_executable($path) };
    (-s $path:expr) => { $crate::fs::is_nonempty_file($path) };
    (-n $str:expr) => { !$str.is_empty() };
    (-z $str:expr) => { $str.is_empty() };
    ($a:expr, ==, $b:expr) => { $crate::utils::str_equals($a, $b) };
    ($a:expr, !=, $b:expr) => { !$crate::utils::str_equals($a, $b) };
    ($a:expr, =~, $b:expr) => { $crate::utils::str_matches($a, $b) };
    ($a:expr, <, $b:expr) => { $a < $b };
    ($a:expr, >, $b:expr) => { $a > $b };
    ($a:expr, -eq, $b:expr) => { $crate::utils::num_eq($a, $b) };
    ($a:expr, -ne, $b:expr) => { !$crate::utils::num_eq($a, $b) };
    ($a:expr, -lt, $b:expr) => { $crate::utils::num_lt($a, $b) };
    ($a:expr, -le, $b:expr) => { $crate::utils::num_lt($a, $b) || $crate::utils::num_eq($a, $b) };
    ($a:expr, -gt, $b:expr) => { $crate::utils::num_gt($a, $b) };
    ($a:expr, -ge, $b:expr) => { $crate::utils::num_gt($a, $b) || $crate::utils::num_eq($a, $b) };

}

#[macro_export]
macro_rules! case {
    ($value:expr, { $($pattern:expr => $body:block),* $(, _ => $default:block)? }) => {
        {
            let val_to_match = $value;
            let mut matched = false;
            $(
                if !matched && $crate::utils::str_matches(val_to_match, $pattern) {
                    matched = true;
                    $body
                }
            )*
            if !matched { $($default)? }
        }
    };
}


#[macro_export]
macro_rules! prompt {
    ($message:expr) => { $crate::utils::prompt_user($message, None) };
    ($message:expr, default: $default:expr) => { $crate::utils::prompt_user($message, Some($default)) };
}
#[macro_export]
macro_rules! confirm {
    ($message:expr) => { $crate::utils::confirm_action($message, None) };
    ($message:expr, default: $default:expr) => { $crate::utils::confirm_action($message, Some($default)) };
}

// --- System & Random Macros ---
#[macro_export]
macro_rules! rand_range {
    ($min:expr, $max:expr) => {{
        use rand::Rng;
        rand::thread_rng().gen_range($min..=$max)
    }};
}
#[macro_export]
macro_rules! clear {
    () => { print!("\x1B[2J\x1B[1;1H"); };
}
#[macro_export]
macro_rules! sleep {
    ($seconds:expr) => { std::thread::sleep(std::time::Duration::from_secs($seconds)) };
    (ms: $ms:expr) => { std::thread::sleep(std::time::Duration::from_millis($ms)) };
}
#[macro_export]
macro_rules! str_line {
    ($char:expr, $count:expr) => { $char.to_string().repeat($count) };
}

// --- Meta & Path Macros ---
#[macro_export]
macro_rules! meta_key {
    ($path:expr, $key:expr) => {
        $crate::fs::extract_meta_from_file($path).get($key).cloned().unwrap_or_default()
    };
}
#[macro_export]
macro_rules! meta_keys {
    ($path:expr, into: $arr_name:expr) => {{
        let meta = $crate::fs::extract_meta_from_file($path);
        let keys: Vec<&str> = meta.keys().map(|s| s.as_str()).collect();
        $crate::context::set_var(&format!("{}_KEYS", $arr_name), &keys.join(" "));
        for (key, value) in meta {
            $crate::context::set_var(&format!("{}_{}", $arr_name, key), &value);
        }
    }};
}
#[macro_export]
macro_rules! path_canon {
    ($path:expr) => { $crate::fs::path_canon($path).unwrap_or_default() };
}
#[macro_export]
macro_rules! path_split {
    ($path:expr, into: $arr_name:expr) => {{
        let parts = $crate::fs::path_split($path);
        for (key, value) in parts {
            $crate::context::set_var(&format!("{}_{}", $arr_name, key), &value);
        }
    }};
}
#[macro_export]
macro_rules! file_in {
    ($file_var:ident in $dir:expr => $body:block) => {
        if let Ok(entries) = std::fs::read_dir($crate::context::expand_vars($dir)) {
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Some(path_str) = entry.path().to_str() {
                         $crate::context::set_var(stringify!($file_var), path_str);
                         $body
                    }
                }
            }
        }
    };
    ($file_var:ident, $content_var:ident in $dir:expr => $body:block) => {
        if let Ok(entries) = std::fs::read_dir($crate::context::expand_vars($dir)) {
            for entry in entries {
                if let Ok(entry) = entry {
                     if let Some(path_str) = entry.path().to_str() {
                        if entry.path().is_file() {
                             $crate::context::set_var(stringify!($file_var), path_str);
                             let content = $crate::fs::read_file(path_str);
                             $crate::context::set_var(stringify!($content_var), &content);
                             $body
                        }
                    }
                }
            }
        }
    };
}

// --- Config Macros ---
#[macro_export]
macro_rules! export {
    () => { $crate::context::export_vars(&$crate::context::get_var("RSB_EXPORT")); };
    ($path:expr) => { $crate::context::export_vars($path); };
}
#[macro_export]
macro_rules! src {
    ($($path:expr),+) => { $crate::load_config!($($path),+); };
}
#[macro_export]
macro_rules! load_config {
    ($($path:expr),+) => { $( $crate::context::load_config_file($path); )+ };
}

// --- Validation Macros ---
#[macro_export]
macro_rules! validate {
    ($condition:expr, $($arg:tt)*) => {
        if !$condition {
            $crate::error!("Validation failed: {}", format!($($arg)*));
            std::process::exit(1);
        }
    };
    ($condition:expr, exit_code: $code:expr, $($arg:tt)*) => {
        if !$condition {
            $crate::error!("Validation failed: {}", format!($($arg)*));
            std::process::exit($code);
        }
    };
}
#[macro_export]
macro_rules! require_file {
    ($path:expr) => { $crate::validate!($crate::fs::is_file($path), "File does not exist: {}", $path); };
}
#[macro_export]
macro_rules! require_dir {
    ($path:expr) => { $crate::validate!($crate::fs::is_dir($path), "Directory does not exist: {}", $path); };
}
#[macro_export]
macro_rules! require_command {
    ($cmd:expr) => { $crate::validate!($crate::os::is_command($cmd), "Command not found: {}", $cmd); };
}
#[macro_export]
macro_rules! require_var {
    ($var:expr) => { $crate::validate!($crate::context::has_var($var), "Required variable not set: {}", $var); };
}

// --- Output Macros ---
#[macro_export]
macro_rules! echo { ($($arg:tt)*) => { println!("{}", $crate::context::expand_vars(&format!($($arg)*))); }; }
#[macro_export]
macro_rules! printf { ($($arg:tt)*) => { print!("{}", $crate::context::expand_vars(&format!($($arg)*))); }; }
#[macro_export]
macro_rules! info { ($($arg:tt)*) => { $crate::utils::glyph_stderr("info", &format!($($arg)*)); }; }
#[macro_export]
macro_rules! okay { ($($arg:tt)*) => { $crate::utils::glyph_stderr("okay", &format!($($arg)*)); }; }
#[macro_export]
macro_rules! warn { ($($arg:tt)*) => { $crate::utils::glyph_stderr("warn", &format!($($arg)*)); }; }
#[macro_export]
macro_rules! error { ($($arg:tt)*) => { $crate::utils::glyph_stderr("error", &format!($($arg)*)); }; }
#[macro_export]
macro_rules! fatal { ($($arg:tt)*) => { $crate::utils::glyph_stderr("fatal", &format!($($arg)*)); }; }
#[macro_export]
macro_rules! debug { ($($arg:tt)*) => { $crate::utils::glyph_stderr("debug", &format!($($arg)*)); }; }
#[macro_export]
macro_rules! trace { ($($arg:tt)*) => { $crate::utils::glyph_stderr("trace", &format!($($arg)*)); }; }

// --- Parameter Expansion Macro ---
#[macro_export]
macro_rules! param {
    ($var:expr) => { $crate::context::get_var($var) };
    ($var:expr, default: $default:expr) => {{
        let val = $crate::context::get_var($var);
        if val.is_empty() { $default.to_string() } else { val }
    }};
    ($var:expr, alt: $alt:expr) => {{
        let val = $crate::context::get_var($var);
        if val.is_empty() { String::new() } else { $alt.to_string() }
    }};

    ($var:expr, sub: $start:expr) => { $crate::utils::str_sub(&$crate::context::get_var($var), $start, None) };
    ($var:expr, sub: $start:expr, $len:expr) => { $crate::utils::str_sub(&$crate::context::get_var($var), $start, Some($len)) };
    ($var:expr, prefix: $pattern:expr) => { $crate::utils::str_prefix(&$crate::context::get_var($var), $pattern, false) };
    ($var:expr, prefix: $pattern:expr, longest) => { $crate::utils::str_prefix(&$crate::context::get_var($var), $pattern, true) };
    ($var:expr, suffix: $pattern:expr) => { $crate::utils::str_suffix(&$crate::context::get_var($var), $pattern, false) };
    ($var:expr, suffix: $pattern:expr, longest) => { $crate::utils::str_suffix(&$crate::context::get_var($var), $pattern, true) };
    ($var:expr, replace: $from:expr => $to:expr) => { $crate::utils::str_replace(&$crate::context::get_var($var), $from, $to, false) };
    ($var:expr, replace: $from:expr => $to:expr, all) => { $crate::utils::str_replace(&$crate::context::get_var($var), $from, $to, true) };
    ($var:expr, upper) => { $crate::utils::str_upper(&$crate::context::get_var($var), true) };
    ($var:expr, lower) => { $crate::utils::str_lower(&$crate::context::get_var($var), true) };
    ($var:expr, upper: first) => { $crate::utils::str_upper(&$crate::context::get_var($var), false) };
    ($var:expr, lower: first) => { $crate::utils::str_lower(&$crate::context::get_var($var), false) };

    ($var:expr, len) => { $crate::context::get_var($var).len() };
}

// --- Benchmarking & Time Macros ---
#[macro_export]
macro_rules! benchmark {
    ($body:block) => {
        {
            let start = std::time::Instant::now();
            $body
            let duration = start.elapsed();
            $crate::info!("Benchmark completed in: {:?}", duration);
            duration
        }
    };
}

#[macro_export]
macro_rules! date {
    () => {
        chrono::Local::now().to_string()
    };
    (iso) => {
        chrono::Local::now().to_rfc3339()
    };
    (epoch) => {
        chrono::Local::now().timestamp().to_string()
    };
    (human) => {
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
    };
    ($format:expr) => {
        chrono::Local::now().format($format).to_string()
    };
}

// --- String Utilities ---
#[macro_export]
macro_rules! str_in {
    ($needle:expr, in: $haystack:expr) => {
        $haystack.contains($needle)
    };
}
#[macro_export]
macro_rules! str_explode {
    ($string:expr, on: $delim:expr, into: $arr_name:expr) => {
        {
            let items: Vec<&str> = $string.split($delim).collect();
            $crate::utils::set_array($arr_name, &items);
        }
    };
}
#[macro_export]
macro_rules! str_trim {
    ($var:expr) => {
        $crate::context::get_var($var).trim().to_string()
    };
}
#[macro_export]
macro_rules! str_len {
    ($var:expr) => {
        $crate::context::get_var($var).len()
    };
}

// --- File System & Temp Macros ---
#[macro_export]
macro_rules! chmod {
    ($path:expr, $mode:expr) => {
        $crate::fs::chmod($path, $mode).ok()
    };
}
#[macro_export]
macro_rules! backup {
    ($path:expr, $suffix:expr) => {
        $crate::fs::backup_file($path, $suffix).ok()
    };
}
#[macro_export]
macro_rules! tmp {
    () => {
        $crate::fs::create_temp_file_path("random")
    };
    ($type:ident) => {
        $crate::fs::create_temp_file_path(stringify!($type))
    };
}

// --- Process Substitution ---
#[macro_export]
macro_rules! cap_stream {
    ($stream:expr) => {
        $crate::fs::capture_stream_to_temp_file(&mut $stream)
    };
}
#[macro_export]
macro_rules! subst {
    ($stream:expr) => {
        $crate::cap_stream!($stream)
    };
}

// --- Math Macros ---
#[macro_export]
macro_rules! math {
    ($expr:expr) => {
        match $crate::math::evaluate_expression($expr) {
            Ok(_) => {},
            Err(e) => {
                $crate::error!("Math expression failed: {}", e);
            }
        }
    };
}

// --- Advanced Sed Macros ---
#[macro_export]
macro_rules! sed_lines {
    ($content:expr, $start:expr, $end:expr) => {
        $crate::streams::Stream::from_string($content).sed_lines($start, $end).to_string()
    };
}

#[macro_export]
macro_rules! sed_around {
    ($content:expr, $pattern:expr, $context:expr) => {
        $crate::streams::Stream::from_string($content).sed_around($pattern, $context).to_string()
    };
}

#[macro_export]
macro_rules! sed_insert {
    ($content:expr, $sentinel:expr, $source:expr) => {{
        match $crate::streams::Stream::from_string($source).sed_insert($content, $sentinel) {
            Ok(stream) => stream.to_string(),
            Err(e) => {
                $crate::error!("sed_insert failed: {}", e);
                std::process::exit(1);
            }
        }
    }};
}

#[macro_export]
macro_rules! sed_template {
    ($content:expr, $sentinel:expr, $source:expr) => {
        $crate::streams::Stream::from_string($source).sed_template($content, $sentinel).to_string()
    };
}

#[macro_export]
macro_rules! sed_replace {
    ($source:expr, $from:expr, $to:expr) => {
        $source.replace($from, $to)
    };
    ($source:expr, $from:expr, $to:expr, all) => {
        $source.replace($from, $to)  // replace() already replaces all occurrences
    };
}

// --- File-based Sed Macros ---
#[macro_export]
macro_rules! sed_lines_file {
    ($path:expr, $start:expr, $end:expr) => {
        $crate::fs::sed_lines_file($path, $start, $end)
    };
}

#[macro_export]
macro_rules! sed_around_file {
    ($path:expr, $pattern:expr, $context:expr) => {
        $crate::fs::sed_around_file($path, $pattern, $context)
    };
}

#[macro_export]
macro_rules! sed_insert_file {
    ($path:expr, $content:expr, $sentinel:expr) => {{
        match $crate::fs::sed_insert_file($path, $content, $sentinel) {
            Ok(_) => {},
            Err(e) => {
                $crate::error!("sed_insert_file failed: {}", e);
                std::process::exit(1);
            }
        }
    }};
}

#[macro_export]
macro_rules! sed_template_file {
    ($path:expr, $content:expr, $sentinel:expr) => {
        $crate::fs::sed_template_file($path, $content, $sentinel)
    };
}

// --- Archive Macros ---
#[macro_export]
macro_rules! tar {
    (create: $archive:expr, $($path:expr),+) => {{
        let paths = vec![$($path),+];
        match $crate::os::create_tar($archive, &paths) {
            result if result.status == 0 => {
                $crate::okay!("Created tar archive: {}", $archive);
            },
            result => {
                $crate::error!("Failed to create tar: {}", result.error);
                std::process::exit(result.status);
            }
        }
    }};
    (extract: $archive:expr) => {{
        match $crate::os::extract_tar($archive, None) {
            result if result.status == 0 => {
                $crate::okay!("Extracted tar archive: {}", $archive);
            },
            result => {
                $crate::error!("Failed to extract tar: {}", result.error);
                std::process::exit(result.status);
            }
        }
    }};
    (extract: $archive:expr, to: $dest:expr) => {{
        match $crate::os::extract_tar($archive, Some($dest)) {
            result if result.status == 0 => {
                $crate::okay!("Extracted tar archive to: {}", $dest);
            },
            result => {
                $crate::error!("Failed to extract tar: {}", result.error);
                std::process::exit(result.status);
            }
        }
    }};
    (list: $archive:expr) => {{
        match $crate::os::list_tar($archive) {
            result if result.status == 0 => result.output,
            result => {
                $crate::error!("Failed to list tar: {}", result.error);
                std::process::exit(result.status);
            }
        }
    }};
}

#[macro_export]
macro_rules! tar_gz {
    (create: $archive:expr, $($path:expr),+) => {{
        let paths = vec![$($path),+];
        match $crate::os::create_tar_gz($archive, &paths) {
            result if result.status == 0 => {
                $crate::okay!("Created tar.gz archive: {}", $archive);
            },
            result => {
                $crate::error!("Failed to create tar.gz: {}", result.error);
                std::process::exit(result.status);
            }
        }
    }};
}

#[macro_export]
macro_rules! zip {
    (create: $archive:expr, $($path:expr),+) => {{
        let paths = vec![$($path),+];
        match $crate::os::create_zip($archive, &paths) {
            result if result.status == 0 => {
                $crate::okay!("Created zip archive: {}", $archive);
            },
            result => {
                $crate::error!("Failed to create zip: {}", result.error);
                std::process::exit(result.status);
            }
        }
    }};
    (extract: $archive:expr) => {{
        match $crate::os::extract_zip($archive, None) {
            result if result.status == 0 => {
                $crate::okay!("Extracted zip archive: {}", $archive);
            },
            result => {
                $crate::error!("Failed to extract zip: {}", result.error);
                std::process::exit(result.status);
            }
        }
    }};
    (extract: $archive:expr, to: $dest:expr) => {{
        match $crate::os::extract_zip($archive, Some($dest)) {
            result if result.status == 0 => {
                $crate::okay!("Extracted zip archive to: {}", $dest);
            },
            result => {
                $crate::error!("Failed to extract zip: {}", result.error);
                std::process::exit(result.status);
            }
        }
    }};
    (list: $archive:expr) => {{
        match $crate::os::list_zip($archive) {
            result if result.status == 0 => result.output,
            result => {
                $crate::error!("Failed to list zip: {}", result.error);
                std::process::exit(result.status);
            }
        }
    }};
}

// Simple pack macro that auto-detects format from extension
#[macro_export]
macro_rules! pack {
    ($archive:expr, $($path:expr),+) => {{
        let archive_path = $archive;
        if archive_path.ends_with(".tar.gz") || archive_path.ends_with(".tgz") {
            $crate::tar_gz!(create: archive_path, $($path),+);
        } else if archive_path.ends_with(".tar") {
            $crate::tar!(create: archive_path, $($path),+);
        } else if archive_path.ends_with(".zip") {
            $crate::zip!(create: archive_path, $($path),+);
        } else {
            $crate::error!("Unsupported archive format: {}", archive_path);
            std::process::exit(1);
        }
    }};
}

// Simple unpack macro that auto-detects format from extension
#[macro_export]
macro_rules! unpack {
    ($archive:expr) => {{
        let archive_path = $archive;
        if archive_path.ends_with(".tar.gz") || archive_path.ends_with(".tgz") || archive_path.ends_with(".tar") {
            $crate::tar!(extract: archive_path);
        } else if archive_path.ends_with(".zip") {
            $crate::zip!(extract: archive_path);
        } else {
            $crate::error!("Unsupported archive format: {}", archive_path);
            std::process::exit(1);
        }
    }};
    ($archive:expr, to: $dest:expr) => {{
        let archive_path = $archive;
        if archive_path.ends_with(".tar.gz") || archive_path.ends_with(".tgz") || archive_path.ends_with(".tar") {
            $crate::tar!(extract: archive_path, to: $dest);
        } else if archive_path.ends_with(".zip") {
            $crate::zip!(extract: archive_path, to: $dest);
        } else {
            $crate::error!("Unsupported archive format: {}", archive_path);
            std::process::exit(1);
        }
    }};
}

// --- System Information Macros ---
#[macro_export]
macro_rules! hostname {
    () => {
        $crate::os::get_hostname()
    };
}

#[macro_export]
macro_rules! user {
    () => {
        $crate::os::get_username()
    };
}

#[macro_export]
macro_rules! home_dir {
    () => {
        $crate::os::get_home_dir()
    };
}

#[macro_export]
macro_rules! current_dir {
    () => {
        $crate::os::get_current_dir()
    };
}

// --- Network Macros ---
#[macro_export]
macro_rules! curl {
    ($url:expr) => {{
        match $crate::os::http_get($url) {
            result if result.status == 0 => result.output,
            result => {
                $crate::error!("curl failed: {}", result.error);
                std::process::exit(result.status);
            }
        }
    }};
    ($url:expr, options: $opts:expr) => {{
        match $crate::os::http_get_with_options($url, $opts) {
            result if result.status == 0 => result.output,
            result => {
                $crate::error!("curl failed: {}", result.error);
                std::process::exit(result.status);
            }
        }
    }};
    (post: $url:expr, data: $data:expr) => {{
        match $crate::os::http_post($url, $data) {
            result if result.status == 0 => result.output,
            result => {
                $crate::error!("curl POST failed: {}", result.error);
                std::process::exit(result.status);
            }
        }
    }};
}

#[macro_export]
macro_rules! get {
    ($url:expr) => {
        $crate::curl!($url)
    };
    ($url:expr, options: $opts:expr) => {
        $crate::curl!($url, options: $opts)
    };
}

// --- Process Management Macros ---
#[macro_export]
macro_rules! pid_of {
    ($process:expr) => {
        $crate::os::pid_of($process)
    };
}

#[macro_export]
macro_rules! process_exists {
    ($process:expr) => {
        $crate::os::process_exists($process)
    };
}

#[macro_export]
macro_rules! kill_pid {
    ($pid:expr) => {{
        match $crate::os::kill_pid($pid, None) {
            result if result.status == 0 => {
                $crate::okay!("Process {} terminated", $pid);
            },
            result => {
                $crate::error!("Failed to kill process {}: {}", $pid, result.error);
                std::process::exit(result.status);
            }
        }
    }};
    ($pid:expr, signal: $sig:expr) => {{
        match $crate::os::kill_pid($pid, Some($sig)) {
            result if result.status == 0 => {
                $crate::okay!("Process {} terminated with {}", $pid, $sig);
            },
            result => {
                $crate::error!("Failed to kill process {}: {}", $pid, result.error);
                std::process::exit(result.status);
            }
        }
    }};
}

#[macro_export]
macro_rules! kill_process {
    ($process:expr) => {{
        match $crate::os::kill_process($process, None) {
            result if result.status == 0 => {
                $crate::okay!("Killed all {} processes", $process);
            },
            result => {
                $crate::error!("Failed to kill {}: {}", $process, result.error);
                std::process::exit(result.status);
            }
        }
    }};
    ($process:expr, signal: $sig:expr) => {{
        match $crate::os::kill_process($process, Some($sig)) {
            result if result.status == 0 => {
                $crate::okay!("Killed all {} processes with {}", $process, $sig);
            },
            result => {
                $crate::error!("Failed to kill {}: {}", $process, result.error);
                std::process::exit(result.status);
            }
        }
    }};
}

// --- Locking Macros ---
#[macro_export]
macro_rules! with_lock {
    ($lock_path:expr => $body:block) => {{
        match $crate::os::create_lock($lock_path) {
            Ok(_) => {
                let result = $body;
                $crate::os::remove_lock($lock_path);
                result
            },
            Err(e) => {
                $crate::error!("Failed to acquire lock: {}", e);
                std::process::exit(1);
            }
        }
    }};
}

#[macro_export]
macro_rules! lock {
    ($lock_path:expr) => {{
        match $crate::os::create_lock($lock_path) {
            Ok(_) => {
                $crate::okay!("Lock acquired: {}", $lock_path);
            },
            Err(e) => {
                $crate::error!("Failed to acquire lock: {}", e);
                std::process::exit(1);
            }
        }
    }};
}

#[macro_export]
macro_rules! unlock {
    ($lock_path:expr) => {
        $crate::os::remove_lock($lock_path);
        $crate::okay!("Lock released: {}", $lock_path);
    };
}

// --- JSON Macros (requires jq) ---
#[macro_export]
macro_rules! json_get {
    ($json:expr, $path:expr) => {
        $crate::os::json_get($json, $path)
    };
}

#[macro_export]
macro_rules! json_get_file {
    ($file:expr, $path:expr) => {
        $crate::os::json_get_file($file, $path)
    };
}

// --- Dictionary Macros ---
#[macro_export]
macro_rules! dict {
    ($path:expr) => {
        $crate::fs::load_dict_from_file($path)
    };
}

#[macro_export]
macro_rules! rand_dict {
    ($arr_name:expr) => {
        $crate::random::get_rand_from_slice(&$crate::utils::get_array($arr_name)).unwrap_or_default()
    };
    ($arr_name:expr, $n:expr) => {
        $crate::rand_dict!($arr_name, $n, " ")
    };
    ($arr_name:expr, $n:expr, $delim:expr) => {{
        let words = $crate::utils::get_array($arr_name);
        if words.is_empty() {
            String::new()
        } else {
            let mut result = Vec::new();
            for _ in 0..$n {
                result.push($crate::random::get_rand_from_slice(&words).unwrap_or_default());
            }
            result.join($delim)
        }
    }};
}

#[macro_export]
macro_rules! gen_dict {
    ($type:ident, $n:expr, into: $arr_name:expr) => {{
        let mut words = Vec::new();
        for _ in 0..$n {
            // A bit of a hack to generate a random word length between 4 and 8
            let len = $crate::rand_range!(4, 8);
            let word = match stringify!($type) {
                "alnum" => $crate::random::get_rand_alnum(len),
                "alpha" => $crate::random::get_rand_alpha(len),
                "hex" => $crate::random::get_rand_hex(len),
                "string" => $crate::random::get_rand_string(len),
                _ => String::new(),
            };
            words.push(word);
        }
        let word_strs: Vec<&str> = words.iter().map(|s| s.as_str()).collect();
        $crate::utils::set_array($arr_name, &word_strs);
    }};
}

// --- Random Data Macros ---
#[macro_export]
macro_rules! rand_alnum {
    ($n:expr) => {
        $crate::random::get_rand_alnum($n)
    };
}
#[macro_export]
macro_rules! rand_alpha {
    ($n:expr) => {
        $crate::random::get_rand_alpha($n)
    };
}
#[macro_export]
macro_rules! rand_hex {
    ($n:expr) => {
        $crate::random::get_rand_hex($n)
    };
}
#[macro_export]
macro_rules! rand_string {
    ($n:expr) => {
        $crate::random::get_rand_string($n)
    };
}
#[macro_export]
macro_rules! rand_uuid {
    () => {
        $crate::random::get_rand_uuid()
    };
}
