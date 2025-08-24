//! examples/showcase.rs

// Using the prelude brings all the essential RSB tools into scope.
use rsb::prelude::*;

fn main() {
    // The bootstrap! macro handles collecting args, loading the environment,
    // and setting up the context all in one go.
    let args = bootstrap!();

    // The dual-dispatch pattern allows for "bootstrap" commands that run
    // before any configuration is loaded. These are for setup, installation, etc.
    if pre_dispatch!(&args, {
        "install" => install_deps,
        "init" => init_project,
        "check" => check_system
    }) {
        return;
    }

    // After pre-dispatch, load configuration files using the `src!` alias.
    info!("Loading configuration...");
    src!("./myapp.conf");

    // The main dispatcher routes commands to their handler functions.
    dispatch!(&args, {
        "build" => build_project,
        "deploy" => deploy,
        "logs" => logs,
        "config" => config,
        "process" => process_data,
        "test" => run_tests,
        "meta-test" => meta_test,
        "date-test" => date_test,
        "path-test" => path_test,
        "file-in-test" => file_in_test,
        "array-test" => array_test,
        "system-test" => system_test,
        "job-test" => job_test,
        "sed-block-test" => sed_block_test,
        "color-test" => color_test,
        "job-test-integration" => job_test_integration,
        "job-test-timeout-integration" => job_test_timeout_integration
    });
}

// --- Pre-Context (Bootstrap) Commands ---

fn install_deps(mut args: Args) -> i32 {
    info!("Installing dependencies...");
    let force = args.has_pop("--force");
    if force {
        warn!("Force installation enabled.");
    }
    cmd!("echo 'Simulating package installation...'")
        .each(|line| okay!("{}", line));
    0
}

fn init_project(args: Args) -> i32 {
    let project_path_str = args.get_or(1, "new-rsb-project");
    let project_path = std::path::Path::new(&project_path_str);

    if let Some(name) = project_path.file_name().and_then(|n| n.to_str()) {
        validate!(is_name(name), "Invalid project name: {}", name);
    } else {
        error!("Invalid project path provided.");
        return 1;
    }

    validate!(!is_entity(&project_path_str), "Project directory already exists: {}", project_path_str);

    info!("Initializing project: {}", project_path_str);
    mkdir_p(&project_path_str);
    let readme_content = format!("# {}\n\nInitialized with RSB.", project_path.file_name().unwrap().to_str().unwrap());
    write_file(&project_path.join("README.md").to_str().unwrap(), &readme_content);

    okay!("Project initialized successfully!");
    0
}

fn check_system(_args: Args) -> i32 {
    info!("Checking system requirements...");
    require_command!("git");
    require_command!("rustc");
    require_command!("cargo");
    okay!("All system requirements satisfied!");
    0
}

// --- Context-Aware Commands ---

fn build_project(mut args: Args) -> i32 {
    require_var!("HOME");
    set_var("PROJECT", "my-app");

    let version = args.has_val("--version").unwrap_or_else(|| "1.0.0".to_string());
    let target = args.get_or(1, "debug");
    let clean = args.has_pop("--clean");
    if let Some(output_dir) = args.get_kv("output") {
        set_var("BUILD_DIR", &output_dir);
    } else {
        set_var("BUILD_DIR", "/tmp/builds");
    }
    if let Some(features) = args.get_array("features") {
        info!("Enabling features: {}", features.join(", "));
    }

    info!("Building $PROJECT v{} for target: {}", version, target);

    if clean {
        warn!("Cleaning workspace...");
        run_cmd("echo 'cargo clean'");
    }

    mkdir_p("$BUILD_DIR");
    let build_log_path = param!("BUILD_DIR", default: "/tmp") + "/build.log";

    pipe!("Compiling module 1...\nCompiling module 2...\n   Finished dev [unoptimized + debuginfo] target(s)")
        .tee(&build_log_path)
        .each(|line| okay!("{}", line));

    okay!("Build successful! Log at {}", build_log_path);
    0
}

fn run_tests(_args: Args) -> i32 {
    info!("Running tests...");
    let results = cmd!("echo 'Running 3 tests\ntest result: ok. 3 passed; 0 failed.'");

    if results.to_string().contains("failed") {
        error!("Tests failed!");
        return 1;
    }

    okay!("All tests passed!");
    0
}

fn deploy(mut args: Args) -> i32 {
    let env = args.get_or(1, "staging");
    let force = args.has_pop("--force");

    case!(env.as_str(), {
        "staging" => {
            info!("Deploying to staging environment.");
        },
        "production" => {
            warn!("Deploying to PRODUCTION environment.");
            if !force {
                error!("Production deploy requires the --force flag.");
                return 1;
            }
            okay!("Force flag provided. Proceeding with production deploy.");
        },
        _ => {
            error!("Unknown environment: {}. Please use 'staging' or 'production'.", env);
            return 1;
        }
    });

    okay!("Deployment to {} successful!", env);
    0
}

fn logs(mut args: Args) -> i32 {
    let log_file = args.has_val("--file").unwrap_or_else(|| "app.log".to_string());
    let errors_only = args.has_pop("--errors");

    pipe!(
        "INFO: Application started\nDEBUG: Connecting to database\nERROR: Failed to connect\nINFO: Retrying..."
    ).to_file(&log_file);

    require_file!(&log_file);
    info!("Reading logs from {}", log_file);

    if errors_only {
        warn!("Showing errors only.");
        cat!(&log_file)
            .grep("ERROR")
            .each(|line| error!("{}", line));
    } else {
        cat!(&log_file)
            .each(|line| echo!("{}", line));
    }
    0
}

fn config(args: Args) -> i32 {
    let action = args.get_or(1, "list");

    match action.as_str() {
        "set" => {
            let key = args.get_or(2, "");
            let value = args.get_or(3, "");
            validate!(!key.is_empty(), "Usage: config set <key> <value>");
            info!("Setting config: {} = {}", key, value);
            set_var(&key, &value);
            save_config_file("./myapp.conf", &[&key]);
            okay!("Configuration saved to ./myapp.conf");
        },
        "get" => {
            let key = args.get_or(2, "");
            validate!(!key.is_empty(), "Usage: config get <key>");
            echo!("{} = {}", key, get_var(&key));
        },
        _ => {
            error!("Unknown config action: {}", action);
            return 1;
        }
    }
    0
}

fn process_data(_args: Args) -> i32 {
    let input_file = "data.csv";
    let output_file = "processed.txt";

    write_file(input_file, "user,active,id\nalice,true,101\nbob,false,102\ncharlie,true,103\nalice,true,104");

    info!("Processing {} -> {}", input_file, output_file);

    let processed_count = cat!(input_file)
        .grep("true")
        .cut(1, ",")
        .unique()
        .sort()
        .tee(output_file)
        .count();

    okay!("Processed {} unique active users.", processed_count);
    info!("Result saved to {}", output_file);

    echo!("\nProcessed Users:");
    cat!(output_file).each(|line| echo!("- {}", line));

    0
}

fn meta_test(args: Args) -> i32 {
    let file_path = args.get_or(1, "meta.txt");
    meta_keys!(&file_path, into: "META");

    echo!("Author: {}", get_var("META_author"));
    echo!("Version: {}", get_var("META_version"));
    0
}

fn array_test(_args: Args) -> i32 {
    info!("Testing array utilities...");
    set_array("MY_ARRAY", &["a", "b", "c"]);
    echo!("Array: $MY_ARRAY");
    echo!("Length: {}", array_length("MY_ARRAY"));
    echo!("Item 1: {}", array_get("MY_ARRAY", 1));
    array_push("MY_ARRAY", "d");
    echo!("Pushed 'd', new array: $MY_ARRAY");
    0
}

fn system_test(_args: Args) -> i32 {
    info!("Testing system utilities...");
    echo!("Line: {}", str_line!('-', 10));
    let num = rand_range!(1, 100);
    echo!("Random number: {}", num);
    validate!(num >= 1 && num <= 100, "Random number out of range");
    0
}


fn date_test(_args: Args) -> i32 {
    info!("Testing date macros...");
    echo!("Default: {}", date!());
    echo!("Epoch: {}", date!(epoch));
    echo!("Human: {}", date!(human));
    echo!("Custom: {}", date!("%Y-%m-%d"));
    let d = benchmark!({
        // sleep for a short duration
        std::thread::sleep(std::time::Duration::from_millis(100));
    });
    info!("Benchmark macro returned duration: {:?}", d);
    0
}

fn path_test(args: Args) -> i32 {
    let path = args.get_or(1, "./README.md");
    info!("Testing path macros for: {}", path);

    let canon_path = path_canon!(&path);
    echo!("Canonical Path: {}", canon_path);

    path_split!(&path, into: "MYPATH");
    echo!("Parent: {}", get_var("MYPATH_parent"));
    echo!("Filename: {}", get_var("MYPATH_file_name"));
    0
}

fn file_in_test(args: Args) -> i32 {
    let dir = args.get_or(1, ".");
    info!("Testing file_in! macro for dir: {}", dir);

    file_in!(file in &dir => {
        echo!("Found file: $file");
    });
    0
}

// --- Test Handlers for New Features ---

fn job_test(args: Args) -> i32 {
    let action = args.get_or(1, "help");
    match action.as_str() {
        "start" => {
            let job_id = job!(background: "sleep 2; echo 'Job Done'");
            echo!("job_id={}", job_id);
        }
        "wait" => {
            let job_id: u32 = args.get_or(2, "0").parse().unwrap_or(0);
            let status = job!(wait: job_id);
            echo!("wait_status={}", status);
        }
        "timeout" => {
            let _job_id: u32 = args.get_or(2, "0").parse().unwrap_or(0);
            // This job runs for 5 seconds, but we time out after 1.
            let long_job_id = job!(background: "sleep 5; echo 'Should not see this'");
            let status = job!(timeout: 1, wait: long_job_id);
            echo!("timeout_status={}", status);
        }
        _ => {
            error!("Unknown job-test action: {}", action);
            return 1;
        }
    }
    0
}

fn sed_block_test(_args: Args) -> i32 {
    let content = "
    # Other file content
    <config>
        <setting>old_value</setting>
    </config>
    # More content
    ";

    // Test replacing content within the block
    let result1 = pipe!(content)
        .sed_block("<config>", "</config>", "s/old_value/new_value/g")
        .to_string();
    echo!("--- Test 1: Replace 'old_value' ---\n{}", result1);

    // Test with no end pattern
    let result2 = pipe!(content)
        .sed_block("<config>", "NO_SUCH_END", "s/old_value/new_value/g")
        .to_string();
    if result2.contains("old_value") {
        echo!("Unclosed block contains: old_value");
    }
    echo!("--- Test 2: No end pattern ---\n{}", result2);

    0
}

fn color_test(_args: Args) -> i32 {
    info!("This is an info message.");
    okay!("This is an okay message.");
    warn!("This is a warning message.");
    error!("This is an error message.");
    fatal!("This is a fatal message.");
    debug!("This is a debug message.");
    trace!("This is a trace message.");
    0
}

fn job_test_integration(_args: Args) -> i32 {
    let job_id = job!(background: "sleep 1; echo 'Job Done'");
    info!("Started job {}", job_id);
    let status = job!(wait: job_id);
    echo!("wait_status={}", status);
    0
}

fn job_test_timeout_integration(_args: Args) -> i32 {
    let job_id = job!(background: "sleep 3; echo 'Should not happen'");
    info!("Started job {}", job_id);
    let status = job!(timeout: 1, wait: job_id);
    echo!("timeout_status={}", status);
    0
}
