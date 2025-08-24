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
        "file-in-test" => file_in_test
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

