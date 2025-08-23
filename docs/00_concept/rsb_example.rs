use rsb::*; // Your RSB framework

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    // Front-load environment and load configs
    for (key, value) in std::env::vars() {
        set_var(&key, &value);
    }
    
    // Set up script awareness
    setup_script_vars(&args);
    
    // Try pre-context dispatch first (bootstrap commands)
    if rsb_pre_dispatch!(args, {
        "install" => install_deps,
        "init" => init_project,
        "check" => check_system,
        "bootstrap" => bootstrap_env
    }) {
        return; // Pre-command was handled
    }
    
    // Load configs after pre-dispatch
    load_config!("/etc/myapp.conf", "$HOME/.myapprc", "./myapp.conf");
    
    // Main context-aware dispatch
    rsb_dispatch!(args, {
        "build" => build_project,
        "deploy" => deploy,
        "logs" => logs,
        "config" => config,
        "process" => process_data,
        "test" => run_tests
    });
}

fn setup_script_vars(args: &[String]) {
    let script_path = &args[0];
    let script_name = std::path::Path::new(script_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("script");
    let script_dir = std::path::Path::new(script_path)
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or(".");
    
    set_var("SCRIPT_NAME", script_name);
    set_var("SCRIPT_PATH", script_path);
    set_var("SCRIPT_DIR", script_dir);
}

// Pre-context bootstrap commands
fn install_deps(mut args: Args) -> i32 {
    echo!("Installing dependencies...");
    
    let force = args.has_pop("--force");
    let quiet = args.has_pop("--quiet");
    
    if let Some(package_file) = args.has_val("--file") {
        echo!("Installing from file: $package_file");
        require_file!(&package_file);
    }
    
    if force {
        echo!("Force installation enabled");
    }
    
    // Simulate installation
    cmd!("echo 'Installing packages...'")
        .each(|line| if !quiet { println!("{}", line); });
    
    0
}

fn init_project(mut args: Args) -> i32 {
    let project_name = args.get_or(1, "new-project");
    let template = args.has_val("--template").unwrap_or("basic".to_string());
    
    validate!(is_name(project_name), "Invalid project name");
    validate!(!is_entity(project_name), "Project directory already exists");
    
    echo!("Initializing project: $project_name with template: $template");
    
    run_cmd(&format!("mkdir -p {}", project_name));
    run_cmd(&format!("echo '# {}' > {}/README.md", project_name, project_name));
    
    echo!("Project initialized successfully!");
    0
}

fn check_system(mut args: Args) -> i32 {
    echo!("Checking system requirements...");
    
    let verbose = args.has_pop("--verbose");
    
    // Check required commands
    let required_commands = ["git", "cargo", "rustc"];
    let mut missing = Vec::new();
    
    for cmd in &required_commands {
        if is_command(cmd) {
            if verbose {
                echo!("✓ $cmd found");
            }
        } else {
            echo!("✗ $cmd not found");
            missing.push(*cmd);
        }
    }
    
    if !missing.is_empty() {
        echo!("Missing required commands: ${missing}");
        return 1;
    }
    
    echo!("All system requirements satisfied!");
    0
}

fn bootstrap_env(mut args: Args) -> i32 {
    echo!("Bootstrapping environment...");
    
    let config_dir = var!("$HOME/.config/$SCRIPT_NAME");
    
    if !is_dir(&config_dir.expand()) {
        run_cmd(&format!("mkdir -p {}", config_dir.expand()));
        echo!("Created config directory: $config_dir");
    }
    
    // Create default config
    let default_config = r#"# Default configuration
PROJECT=my-app
VERSION=1.0.0
BUILD_DIR=/tmp/builds
DEPLOY_HOSTS=(staging.com production.com)
"#;
    
    let config_file = var!("$config_dir/config.conf");
    if !is_file(&config_file.expand()) {
        write_file(&config_file.expand(), default_config);
        echo!("Created default config: $config_file");
    }
    
    echo!("Environment bootstrapped successfully!");
    0
}

// Context-aware commands
fn build_project(mut args: Args) -> i32 {
    require_var!("PROJECT");
    
    // Parse sophisticated arguments
    let version = args.has_val("--version").unwrap_or_else(|| get_var("VERSION"));
    let target = args.get_or(1, "debug");
    let clean = args.has_pop("--clean");
    let verbose = args.has_pop("--verbose");
    
    // Parse key-value arguments
    if let Some(output_dir) = args.get_kv("output") {
        set_var("BUILD_DIR", &output_dir);
    }
    
    // Parse array arguments
    if let Some(features) = args.get_array("features") {
        set_array("BUILD_FEATURES", &features.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    }
    
    set_var("VERSION", &version);
    array!("TARGETS", ["debug", "release", "test"]);
    
    // Validate target
    let valid_targets = get_array("TARGETS");
    validate!(valid_targets.contains(&target.to_string()), 
              format!("Invalid target: {}. Valid: {}", target, valid_targets.join(", ")));
    
    echo!("Building $PROJECT v$VERSION for target: $target");
    
    if clean {
        echo!("Cleaning workspace...");
        cmd!("cargo clean")
            .each(|line| if verbose { println!("  {}", line); });
    }
    
    // Build with features if specified
    let mut build_cmd = format!("cargo build --{}", target);
    if has_var("BUILD_FEATURES_LENGTH") {
        let features = get_array("BUILD_FEATURES");
        build_cmd.push_str(&format!(" --features {}", features.join(",")));
    }
    
    echo!("Running: $build_cmd");
    
    let build_output = cmd!(&build_cmd)
        .tee(&var!("$BUILD_DIR/build-$VERSION.log").expand())
        .grep("error")
        .each(|line| eprintln!("ERROR: {}", line))
        .count();
    
    if build_output > 0 {
        echo!("Build failed with $build_output errors");
        return 1;
    }
    
    echo!("Build successful!");
    0
}

fn run_tests(mut args: Args) -> i32 {
    let pattern = args.has_val("--pattern");
    let parallel = args.has_pop("--parallel");
    let coverage = args.has_pop("--coverage");
    
    // Parse test configuration
    if let Some(test_dirs) = args.get_array("dirs") {
        set_array("TEST_DIRS", &test_dirs.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    }
    
    let mut test_cmd = "cargo test".to_string();
    
    if let Some(p) = pattern {
        test_cmd.push_str(&format!(" {}", p));
    }
    
    if parallel {
        test_cmd.push_str(" --test-threads 4");
    }
    
    echo!("Running tests: $test_cmd");
    
    let test_results = cmd!(&test_cmd)
        .tee("test-results.log")
        .grep("test result:")
        .head(1)
        .first()
        .cloned()
        .unwrap_or_default();
    
    if test_results.contains("FAILED") {
        echo!("Tests failed!");
        return 1;
    }
    
    echo!("All tests passed!");
    
    if coverage {
        echo!("Generating coverage report...");
        cmd!("cargo tarpaulin --out Html")
            .each(|line| println!("{}", line));
    }
    
    0
}

fn deploy(mut args: Args) -> i32 {
    require_var!("VERSION");
    require_var!("PROJECT");
    
    let env = args.get_or(1, "staging");
    let force = args.has_pop("--force");
    let dry_run = args.has_pop("--dry-run");
    
    // Validate environment
    let valid_envs = get_array("DEPLOY_HOSTS");
    if !valid_envs.is_empty() {
        validate!(valid_envs.iter().any(|h| h.contains(env)), 
                  format!("Invalid environment: {}. Valid: {}", env, valid_envs.join(", ")));
    }
    
    if env == "production" && !force {
        validate!(false, "Production deploy requires --force flag", 2);
    }
    
    echo!("Deploying $PROJECT v$VERSION to $env");
    
    if dry_run {
        echo!("DRY RUN: Would deploy to $env");
        return 0;
    }
    
    // Generate and execute deployment
    cat!("deploy-template.sh")
        .sed("{{VERSION}}", &get_var("VERSION"))
        .sed("{{ENV}}", env)
        .sed("{{PROJECT}}", &get_var("PROJECT"))
        .to_file(&var!("deploy-$env.sh").expand());
    
    echo!("Generated deployment script: deploy-$env.sh");
    
    let deploy_result = cmd!(&var!("bash deploy-$env.sh").expand())
        .tee(&var!("deploy-$env.log").expand())
        .grep("ERROR")
        .count();
    
    if deploy_result > 0 {
        echo!("Deployment failed with errors");
        return 1;
    }
    
    echo!("Deployment successful!");
    0
}

fn logs(mut args: Args) -> i32 {
    let log_file = args.has_val("--file")
        .unwrap_or_else(|| var!("${LOG_DIR:-/var/log}/${PROJECT:-app}.log").expand());
    
    let follow = args.has_pop("--follow");
    let errors_only = args.has_pop("--errors");
    let lines: usize = args.has_val("--lines")
        .and_then(|s| s.parse().ok())
        .unwrap_or(50);
    
    require_file!(&log_file);
    
    echo!("Reading logs from: $log_file");
    
    if follow {
        echo!("Following log file (Ctrl+C to stop)...");
        // Simulate tail -f
        cmd!(&format!("tail -f {}", log_file))
            .each(|line| println!("{}", line));
    } else if errors_only {
        cat!(&log_file)
            .grep("ERROR")
            .tail(lines)
            .each(|line| println!("ERROR: {}", line));
    } else {
        cat!(&log_file)
            .tail(lines)
            .each(|line| println!("{}", line));
    }
    
    0
}

fn config(mut args: Args) -> i32 {
    match args.get(1) {
        "set" => {
            let key = args.get(2);
            let value = args.get(3);
            validate!(!key.is_empty() && !value.is_empty(), 
                     "Usage: config set <key> <value>");
            
            set_var(key, value);
            echo!("Set $key = $value");
            save_config_file("./myapp.conf", &[key]);
        },
        "get" => {
            let key = args.get(2);
            validate!(!key.is_empty(), "Usage: config get <key>");
            
            if has_var(key) {
                println!("{}", get_var(key));
            } else {
                echo!("Variable $key not set");
                return 1;
            }
        },
        "validate" => {
            echo!("Validating configuration...");
            require_var!("PROJECT");
            require_var!("VERSION");
            echo!("Configuration is valid!");
        },
        _ => {
            echo!("Usage: config set|get|validate <args>");
            return 1;
        }
    }
    
    0
}

fn process_data(mut args: Args) -> i32 {
    let input_file = args.get_or(1, "data.csv");
    let output_file = args.get_or(2, "processed.txt");
    
    require_file!(input_file);
    
    let format = args.has_val("--format").unwrap_or("txt".to_string());
    let separator = args.has_val("--sep").unwrap_or(",".to_string());
    
    echo!("Processing $input_file -> $output_file (format: $format)");
    
    let processed_count = cat!(input_file)
        .filter(|line| !line.trim().is_empty())
        .cut(2, &separator)
        .filter(|line| !line.is_empty())
        .sort()
        .uniq()
        .tee(output_file)
        .count();
    
    echo!("Processed $processed_count unique records");
    0
}

// Usage examples:
// ./tool install --force --file packages.txt
// ./tool init myproject --template rust
// ./tool check --verbose
// ./tool bootstrap
//
// ./tool build --version 2.0.0 --clean features=logging,auth output=/tmp/mybuild
// ./tool test --pattern integration --coverage --parallel
// ./tool deploy production --force
// ./tool logs --errors --lines 100 --file /var/log/myapp.log
// ./tool config set DATABASE_URL "postgres://localhost/mydb"
// ./tool config validate
// ./tool process input.csv output.json --format json --sep "|"
//
// Built-in commands:
// ./tool help
// ./tool inspect
// ./tool stackuse rsb::*; // Your RSB framework

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    // Front-load environment and load configs
    for (key, value) in std::env::vars() {
        set_var(&key, &value);
    }
    
    // Load multiple config files (last wins)
    load_config!("/etc/myapp.conf", "$HOME/.myapprc", "./myapp.conf");
    
    // Set up script awareness
    setup_script_vars(&args);
    
    rsb_dispatch!(args, {
        "build" => build_project,
        "deploy" => deploy,
        "logs" => logs,
        "config" => config,
        "process" => process_data
    });
}

fn setup_script_vars(args: &[String]) {
    let script_path = &args[0];
    let script_name = std::path::Path::new(script_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("script");
    let script_dir = std::path::Path::new(script_path)
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or(".");
    
    set_var("SCRIPT_NAME", script_name);
    set_var("SCRIPT_PATH", script_path);
    set_var("SCRIPT_DIR", script_dir);
}

fn build_project(args: Args) -> i32 {
    // Set some context variables
    set_var("PROJECT", "my-app");
    set_var("VERSION", args.get_or(1, "1.0.0"));
    set_var("BUILD_DIR", "/tmp/builds");
    
    // Set up supported targets as an array
    array!("TARGETS", ["debug", "release", "test"]);
    
    let target = args.get_or(2, "debug");
    
    // Validate target
    let valid_targets = get_array("TARGETS");
    if !valid_targets.contains(&target.to_string()) {
        echo!("Invalid target: $2. Valid targets: ${TARGETS}");
        return 1;
    }
    
    echo!("Building $PROJECT v$VERSION for target: $target");
    
    if args.has("--clean") {
        echo!("Cleaning workspace...");
        cmd!("cargo clean").each(|line| println!("  {}", line));
    }
    
    // Stream-based build log processing
    let build_output = cmd!(&var!("cargo build --${target}").expand())
        .tee(&var!("$BUILD_DIR/build-$VERSION.log").expand())
        .grep("error")
        .each(|line| eprintln!("ERROR: {}", line))
        .count();
    
    if build_output > 0 {
        echo!("Build failed with $build_output errors");
        return 1;
    }
    
    echo!("Build successful!");
    0
}

fn deploy(args: Args) -> i32 {
    let env = args.get_or(1, "staging");
    
    if !has_var("VERSION") {
        eprintln!("No version set! Run build first.");
        return 1;
    }
    
    // Load deployment hosts from config
    let deploy_hosts = get_array("DEPLOY_HOSTS");
    if deploy_hosts.is_empty() {
        array!("DEPLOY_HOSTS", ["staging.example.com", "prod.example.com"]);
    }
    
    set_var("DEPLOY_ENV", env);
    echo!("Deploying $PROJECT v$VERSION to $DEPLOY_ENV");
    
    // Process deployment in a stream
    let deploy_result = cat!("deploy-template.sh")
        .sed("{{VERSION}}", &get_var("VERSION"))
        .sed("{{ENV}}", env)
        .to_file(&var!("deploy-$DEPLOY_ENV.sh").expand())
        .to_string();
    
    echo!("Generated deployment script: deploy-$DEPLOY_ENV.sh");
    
    if args.has("--execute") {
        let exit_code = run_cmd(&var!("bash deploy-$DEPLOY_ENV.sh").expand());
        if exit_code.is_empty() {
            return 1;
        }
    }
    
    0
}

fn logs(args: Args) -> i32 {
    set_var("LOG_DIR", "/var/log");
    set_var("APP_NAME", &get_var("PROJECT"));
    
    let log_file = args.get_or(1, "${LOG_DIR}/${APP_NAME}.log");
    let expanded_path = var!(log_file);
    
    echo!("Reading logs from: $expanded_path");
    
fn logs(args: Args) -> i32 {
    set_var("LOG_DIR", "/var/log");
    set_var("APP_NAME", &get_var("PROJECT"));
    
    let log_file = args.get_or(1, "${LOG_DIR}/${APP_NAME}.log");
    let expanded_path = var!(log_file);
    
    echo!("Reading logs from: $expanded_path");
    
    if args.has("--errors") {
        // Complex pipeline: errors in last hour, sorted by frequency
        cat!(&expanded_path.expand())
            .grep("ERROR")
            .grep(&chrono::Utc::now().format("%Y-%m-%d %H").to_string())
            .cut(4, " ")  // Extract error message part
            .sort()
            .uniq()
            .head(10)
            .each(|line| println!("ERROR: {}", line));
    } else if args.has("--follow") {
        // Tail -f equivalent 
        let tail_count: usize = args.get_or(2, "20").parse().unwrap_or(20);
        cat!(&expanded_path.expand())
            .tail(tail_count)
            .each(|line| println!("{}", line));
    } else {
        // Default: last 50 lines
        cat!(&expanded_path.expand())
            .tail(50)
            .each(|line| println!("{}", line));
    }
    
    0
}

fn config(args: Args) -> i32 {
    match args.get(1) {
        "set" => {
            let key = args.get(2);
            let value = args.get(3);
            if !key.is_empty() && !value.is_empty() {
                set_var(key, value);
                echo!("Set $key = $value");
                
                // Save to local config
                save_config_file("./myapp.conf", &[key]);
            } else {
                echo!("Usage: config set <key> <value>");
                return 1;
            }
        },
        "get" => {
            let key = args.get(2);
            if !key.is_empty() {
                if has_var(key) {
                    println!("{}", get_var(key));
                } else {
                    echo!("Variable $key not set");
                    return 1;
                }
            } else {
                echo!("Usage: config get <key>");
                return 1;
            }
        },
        "list" => {
            // This would require extending Context to list all vars
            echo!("Listing all configuration variables:");
            echo!("PROJECT=$PROJECT");
            echo!("VERSION=$VERSION");
            echo!("BUILD_DIR=$BUILD_DIR");
        },
        "array" => {
            let action = args.get(2);
            let key = args.get(3);
            
            match action {
                "set" => {
                    let items: Vec<&str> = args.all()[4..].iter().map(|s| s.as_str()).collect();
                    set_array(key, &items);
                    echo!("Set array $key = [${items}]");
                },
                "get" => {
                    let items = get_array(key);
                    println!("{}", items.join(" "));
                },
                "push" => {
                    let item = args.get(4);
                    push_array(key, item);
                    echo!("Added '$item' to array $key");
                },
                _ => {
                    echo!("Usage: config array set|get|push <key> [items...]");
                    return 1;
                }
            }
        },
        _ => {
            echo!("Usage: config set|get|list|array <args>");
            return 1;
        }
    }
    
    0
}

fn process_data(args: Args) -> i32 {
    let input_file = args.get_or(1, "data.csv");
    let output_file = args.get_or(2, "processed.txt");
    
    echo!("Processing $input_file -> $output_file");
    
    // Complex data processing pipeline
    let processed_count = cat!(&input_file)
        .grep("active")                    // Only active records
        .cut(2, ",")                       // Get second column  
        .filter(|line| !line.is_empty())   // Remove empty lines
        .map(|line| line.to_uppercase())   // Convert to uppercase
        .sort()                            // Sort alphabetically
        .uniq()                            // Remove duplicates
        .tee(&output_file)                 // Write to file and continue
        .count();                          // Count final results
    
    echo!("Processed $processed_count unique records");
    echo!("Output saved to $output_file");
    
    0
}

// Usage examples:
// ./tool build 2.0.0 release --clean
// ./tool deploy production --execute
// ./tool logs --errors
// ./tool logs --follow 100
// ./tool config set DATABASE_URL "postgres://..."
// ./tool config array set DEPLOY_HOSTS staging.com prod.com backup.com
// ./tool config array push DEPLOY_HOSTS "new-server.com"
// ./tool process_data input.csv output.txt") {
        // Complex pipeline: errors in last hour, sorted by frequency
        cat!(&expanded_path.expand())
            .grep("ERROR")
            .grep(&chrono::Utc::now().format("%Y-%m-%d %H").to_string())
            .cut(4, " ")  // Extract error message part
            .sort()
            .uniq()
            .head(10)
            .each(|line| println!("ERROR: {}", line));
    } else if args.has("--follow") {
        // Tail -f equivalent 
        let tail_count: usize = args.get_or(2, "20").parse().unwrap_or(20);
        cat!(&expanded_path.expand())
            .tail(tail_count)
            .each(|line| println!("{}", line));
    } else {
        // Default: last 50 lines
        cat!(&expanded_path.expand())
            .tail(50)
            .each(|line| println!("{}", line));
    }
    
    0
}

fn config(args: Args) -> i32 {
    match args.get(1) {
        "set" => {
            let key = args.get(2);
            let value = args.get(3);
            if !key.is_empty() && !value.is_empty() {
                set_var(key, value);
                echo!("Set $key = $value");
                
                // Save to local config
                save_config_file("./myapp.conf", &[key]);
            } else {
                echo!("Usage: config set <key> <value>");
                return 1;
            }
        },
        "get" => {
            let key = args.get(2);
            if !key.is_empty() {
                if has_var(key) {
                    println!("{}", get_var(key));
                } else {
                    echo!("Variable $key not set");
                    return 1;
                }
            } else {
                echo!("Usage: config get <key>");
                return 1;
            }
        },
        "list" => {
            // This would require extending Context to list all vars
            echo!("Listing all configuration variables:");
            echo!("PROJECT=$PROJECT");
            echo!("VERSION=$VERSION");
            echo!("BUILD_DIR=$BUILD_DIR");
        },
        "array" => {
            let action = args.get(2);
            let key = args.get(3);
            
            match action {
                "set" => {
                    let items: Vec<&str> = args.all()[4..].iter().map(|s| s.as_str()).collect();
                    set_array(key, &items);
                    echo!("Set array $key = [${items}]");
                },
                "get" => {
                    let items = get_array(key);
                    println!("{}", items.join(" "));
                },
                "push" => {
                    let item = args.get(4);
                    push_array(key, item);
                    echo!("Added '$item' to array $key");
                },
                _ => {
                    echo!("Usage: config array set|get|push <key> [items...]");
                    return 1;
                }
            }
        },
        _ => {
            echo!("Usage: config set|get|list|array <args>");
            return 1;
        }
    }
    
    0
}

fn process_data(args: Args) -> i32 {
    let input_file = args.get_or(1, "data.csv");
    let output_file = args.get_or(2, "processed.txt");
    
    echo!("Processing $input_file -> $output_file");
    
    // Complex data processing pipeline
    let processed_count = cat!(&input_file)
        .grep("active")                    // Only active records
        .cut(2, ",")                       // Get second column  
        .filter(|line| !line.is_empty())   // Remove empty lines
        .map(|line| line.to_uppercase())   // Convert to uppercase
        .sort()                            // Sort alphabetically
        .uniq()                            // Remove duplicates
        .tee(&output_file)                 // Write to file and continue
        .count();                          // Count final results
    
    echo!("Processed $processed_count unique records");
    echo!("Output saved to $output_file");
    
    0
}

// Usage examples:
// ./tool build 2.0.0 release --clean
// ./tool deploy production --execute
// ./tool logs --errors
// ./tool logs --follow 100
// ./tool config set DATABASE_URL "postgres://..."
// ./tool config array set DEPLOY_HOSTS staging.com prod.com backup.com
// ./tool config array push DEPLOY_HOSTS "new-server.com"
// ./tool process_data input.csv output.txt") {
        // Chain string operations like bash pipeline
        let errors = content
            .grep("ERROR")
            .join("\n")
            .head(20);
        
        for line in errors {
            println!("{}", line);
        }
    } else {
        let tail_count = args.get_or(1, "50");
        set_var("TAIL_COUNT", tail_count);
        
        let lines = content.tail(get_var("TAIL_COUNT").parse().unwrap_or(50));
        for line in lines {
            println!("{}", line);
        }
    }
}

fn config(args: Args) {
    match args.get(0) {
        "set" => {
            let key = args.get(1);
            let value = args.get(2);
            if !key.is_empty() && !value.is_empty() {
                set_var(key, value);
                println!("Set {} = {}", key, value);
            }
        },
        "get" => {
            let key = args.get(1);
            if !key.is_empty() {
                println!("{}", get_var(key));
            }
        },
        _ => {
            println!("Usage: config set|get <key> [value]");
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    // Initialize some defaults
    set_var("HOME", std::env::var("HOME").unwrap_or_default());
    set_var("USER", std::env::var("USER").unwrap_or_default());
    
    rsb_dispatch!(args, {
        "build" => build_project,
        "deploy" => deploy,
        "logs" => logs,
        "config" => config
    });
}

// Usage examples:
// ./tool build 2.0.0 release --clean
// ./tool deploy production  
// ./tool logs --errors
// ./tool config set DATABASE_URL "postgres://..."
// ./tool config get DATABASE_URL