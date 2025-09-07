use rsb::preamble::*;

// RSB port of func.sh - Function extraction and management utility
// Maintains same command interface as the original BashFX utility

fn main() {
    let args = bootstrap!();
    
    // BashFX-style context setup
    set_var("FUNC_NAME", "APP_FUNC");
    set_var("APP_FUNC_VERSION", "0.4.0");
    set_var("APP_FUNC", get_var("SCRIPT_PATH"));
    set_var("FUNC_PREF", "funcx");
    
    // Configuration defaults
    set_var("SAFE_MODE", "1");
    set_var("QUIET_MODE", "0");
    
    // Options (populated by _parse_options)
    set_var("opt_yes", "0");
    set_var("opt_force", "0");
    set_var("opt_alias", "");
    set_var("opt_bash", "0");
    
    // Pre-dispatch for independent commands
    let handled = pre_dispatch!(&args, {
        "help" => do_usage,
        "version" => do_version,
        "" => do_usage_exit
    });
    if handled {
        return;
    }
    
    // Parse global options
    let cleaned_args = _parse_options(Args::new(&args));
    
    let cleaned_vec = cleaned_args.all().to_vec();
    dispatch!(&cleaned_vec, {
        "copy" => do_copy,
        "insert" => do_insert,
        "done" => do_done,
        "clean" => do_clean,
        "spy" => do_spy,
        "extract" => do_extract,
        "check" => do_check,
        "meta" => do_meta,
        "flag" => do_flag,
        "point" => do_point,
        "where" => do_where,
        "ls" => do_ls,
        "find" => do_find
    });
}

fn do_usage(_args: Args) -> i32 {
    let version = get_var("APP_FUNC_VERSION");
    echo!(r#"func v{}
Usage: func <command> [args...]

A powerful, safety-conscious utility for the FIIP workflow.

WORKFLOW COMMANDS:
  copy <func> <src> [--alias <new>] [-f|--force]
  insert <new_func> <src> [-y|--yes] [-f|--force]
  done <func_name>
  clean [-f|--force]

UTILITY COMMANDS:
  spy <func> <src>
  extract <func> <src>
  check <func_name>
  meta <func_file_in_func_dir>
  flag <func> <new> <src>
  point <new> <src>
  where <func> <src> [--bash]
  ls <src> [--bash]
  find <pattern> <src> [--bash]

GLOBAL FLAGS:
  -y, --yes          Answers 'yes' to confirmation prompts.
  -f, --force        Overrides safety guards (e.g., file overwrites).
  -q, --quiet        Suppresses all stderr output.
  --alias <name>     Provides a custom name for 'copy' command.
  --bash             Treats source file as a shell script, bypassing validation.
  -h, --help         Displays this help message.
  --version          Displays version information.
"#, version);
    0
}

fn do_version(_args: Args) -> i32 {
    let version = get_var("APP_FUNC_VERSION");
    echo!(r#"Func, version {},(RSB Utility) (requires Rust/RSB)
Copyright (C) 2025, Qodeninja. Qodeninja Software.
License GPLv3+: GNU GPL version 3 or later <http://gnu.org/licenses/gpl.html>

This is free software; you are free to change and redistribute it.
There is NO WARRANTY, to the extent permitted by law.

A commercial license with different terms, including no copyleft restrictions,
is available separately for business use. 

For more information, please visit github.com/qodeninja/fx-func.
"#, version);
    0
}

fn do_usage_exit(_args: Args) -> i32 {
    do_usage(_args);
    1  // Return error code, dispatch! will exit
}

// === OPTION PARSING ===

fn _parse_options(args: Args) -> Args {
    let mut cleaned_args = Vec::new();
    let all_args = args.all();
    let mut i = 0;
    
    while i < all_args.len() {
        let arg = &all_args[i];
        match arg.as_str() {
            "-y" | "--yes" => {
                set_var("opt_yes", "1");
                i += 1;
            },
            "-f" | "--force" => {
                set_var("opt_force", "1");
                i += 1;
            },
            "-q" | "--quiet" => {
                set_var("QUIET_MODE", "1");
                i += 1;
            },
            "--alias" => {
                if i + 1 < all_args.len() {
                    set_var("opt_alias", &all_args[i + 1]);
                    i += 2;
                } else {
                    fatal!("--alias requires a value");
                }
            },
            "--bash" => {
                set_var("opt_bash", "1");
                i += 1;
            },
            "-h" | "--help" => {
                cleaned_args.push("help".to_string());
                i += 1;
            },
            "--version" => {
                cleaned_args.push("version".to_string());
                i += 1;
            },
            _ if arg.starts_with("-") => {
                fatal!("Unknown option: {}", arg);
            },
            _ => {
                cleaned_args.push(arg.clone());
                i += 1;
            }
        }
    }
    
    Args::new(&cleaned_args)
}

// === MID-LEVEL FUNCTIONS ===

fn _is_valid_shell_source(file_path: &str) -> bool {
    if get_var("opt_bash") == "1" {
        return true;
    }
    
    if !test!(-f file_path) {
        return false;
    }
    
    // Check extension
    if file_path.ends_with(".sh") || file_path.ends_with(".bash") || 
       file_path.ends_with(".func") || file_path.ends_with(".fx") {
        return true;
    }
    
    if file_path.ends_with(".log") || file_path.ends_with(".txt") || file_path.ends_with(".md") {
        return false;
    }
    
    // Check shebang
    let first_line = run!("head -n 1 '{}'", file_path);
    first_line.contains("bash")
}

fn _extract_function_body(func_name: &str, src_file: &str) -> String {
    let awk_script = format!(r#"
        BEGIN {{ in_func = 0; brace_level = 0; }}
        $0 ~ "^[[:space:]]*{0}[[:space:]]*\\([[:space:]]*\\)[[:space:]]*\\{{" {{
            if (in_func == 0) {{
                in_func = 1;
                for (i = 1; i <= length($0); ++i) {{ if (substr($0, i, 1) == "{{") brace_level++; }}
                print $0; next;
            }}
        }}
        in_func == 1 {{
            print $0;
            for (i = 1; i <= length($0); ++i) {{
                if (substr($0, i, 1) == "{{") brace_level++;
                else if (substr($0, i, 1) == "}}") brace_level--;
            }}
            if (brace_level == 0) in_func = 0;
        }}
    "#, func_name);
    
    run!("awk '{}' '{}'", awk_script, src_file)
}

fn _find_function_line(func_name: &str, src_file: &str) -> String {
    let pattern = format!(r"^[[:space:]]*{}[[:space:]]*\([[:space:]]*\)[[:space:]]*\{{", func_name);
    run!("grep -n -E '{}' '{}' | cut -d: -f1", pattern, src_file)
}

fn _parse_meta_header(field: &str, file_path: &str) -> String {
    let pattern = format!(r".*{}:\([^ |]*\).*", field);
    run!("grep '^# FUNC_META' '{}' | sed -n 's/{}/\\1/p'", file_path, pattern)
}

fn _checksum(content: &str) -> String {
    if is_command("sha256sum") {
        run!("echo '{}' | sha256sum | awk '{{print $1}}'", content)
    } else {
        run!("echo '{}' | md5sum | awk '{{print $1}}'", content)
    }
}

fn _safe_cp(src: &str, dest: &str) {
    let result = shell!("cp '{}' '{}'", src, dest);
    if result.status != 0 {
        fatal!("Filesystem operation failed: cp '{}' to '{}'", src, dest);
    }
}

fn _safe_mv(src: &str, dest: &str) {
    let result = shell!("mv '{}' '{}'", src, dest);
    if result.status != 0 {
        fatal!("Filesystem operation failed: mv '{}' to '{}'", src, dest);
    }
}

fn _safe_rm(args: &str) {
    let result = shell!("rm {}", args);
    if result.status != 0 {
        fatal!("Filesystem operation failed: rm {}", args);
    }
}

fn _find_flag_line(pattern: &str, src_file: &str) -> String {
    let marker = format!("# FUNC_INSERT ./func/{}.edit.sh", pattern);
    run!("grep -n '{}' '{}' | cut -d: -f1", marker, src_file)
}

// === WORKFLOW COMMAND FUNCTIONS ===

fn do_copy(args: Args) -> i32 {
    let func_name = args.get_or(1, "");
    let src_path = args.get_or(2, "");
    
    if func_name.is_empty() || src_path.is_empty() {
        error!("Usage: func copy <func> <src> [--alias <new>] [-f|--force]");
        return 1;
    }
    
    if !_is_valid_shell_source(&src_path) {
        fatal!("Source file '{}' does not appear to be a valid shell script. Use --bash to override.", src_path);
    }
    
    let func_body = _extract_function_body(&func_name, &src_path);
    if func_body.trim().is_empty() {
        fatal!("Function '{}' not found in '{}'.", func_name, src_path);
    }
    
    mkdir_p("./func");
    
    let new_func_name = if get_var("opt_alias").is_empty() {
        func_name.to_string()
    } else {
        get_var("opt_alias")
    };
    
    let orig_file = format!("./func/{}.orig.sh", new_func_name);
    let edit_file = format!("./func/{}.edit.sh", new_func_name);
    
    if (test!(-f &orig_file) || test!(-f &edit_file)) && get_var("opt_force") == "0" {
        error!("Target files already exist. Use --force to overwrite.");
        if test!(-f &orig_file) {
            error!("  - Exists: {}", orig_file);
        }
        if test!(-f &edit_file) {
            error!("  - Exists: {}", edit_file);
        }
        return 1;
    }
    
    write_file(&orig_file, &func_body);
    let renamed_body = sed_replace!(&func_body, &func_name, &new_func_name);
    write_file(&edit_file, &renamed_body);
    
    let src_sum = _checksum(&read_file(&src_path));
    let orig_sum = _checksum(&func_body);
    let real_src_path = run!("realpath '{}'", src_path);
    let header = format!("# FUNC_META | src:{} | src_sum:{} | orig:{} | edit:{} | orig_sum:{}", 
                        real_src_path, src_sum, func_name, new_func_name, orig_sum);
    
    // Insert header at the beginning of both files
    let orig_with_header = format!("{}\n{}", header, func_body);
    let edit_with_header = format!("{}\n{}", header, renamed_body);
    write_file(&orig_file, &orig_with_header);
    write_file(&edit_file, &edit_with_header);
    
    info!("Created reference file: '{}'", orig_file);
    info!("Created working file:   '{}'", edit_file);
    
    0
}

fn do_insert(args: Args) -> i32 {
    let new_func_name = args.get_or(1, "");
    let src_path = args.get_or(2, "");
    
    if new_func_name.is_empty() || src_path.is_empty() {
        error!("Usage: func insert <new_func> <src> [-y|--yes] [-f|--force]");
        return 1;
    }
    
    let edit_file = format!("./func/{}.edit.sh", new_func_name);
    let marker_pattern = format!("# FUNC_INSERT {}", edit_file);
    
    if !test!(-f &edit_file) {
        fatal!("Function file not found: '{}'.", edit_file);
    }
    
    let marker_check = run!("grep -q '{}' '{}'; echo $?", marker_pattern, src_path);
    if marker_check.trim() != "0" {
        fatal!("FUNC_INSERT marker not found in '{}'.", src_path);
    }
    
    if get_var("SAFE_MODE") == "1" {
        let meta_src = _parse_meta_header("src", &edit_file);
        let real_src_path = run!("realpath '{}'", src_path);
        
        if meta_src != real_src_path {
            let meta_src_sum = _parse_meta_header("src_sum", &edit_file);
            let current_src_sum = _checksum(&read_file(&src_path));
            
            if meta_src_sum == current_src_sum {
                if get_var("opt_yes") == "1" || _yes_prompt(&format!("Warning: Source path mismatch, but checksums match. Update metadata in '{}'?", edit_file)) {
                    let updated_content = sed_replace!(&read_file(&edit_file), &meta_src, &real_src_path);
                    write_file(&edit_file, &updated_content);
                } else {
                    fatal!("Abort.");
                }
            } else {
                fatal!("SAFE_MODE Abort. Source file path and checksum both mismatch.");
            }
        }
    }
    
    let orig_backup = format!("{}.orig", src_path);
    if test!(-f &orig_backup) {
        if get_var("opt_force") == "1" {
            _safe_mv(&orig_backup, &format!("{}.orig.0", src_path));
            _safe_cp(&src_path, &orig_backup);
        } else if get_var("opt_yes") == "0" {
            fatal!("Backup file '{}' already exists. Use --yes to proceed without creating a new backup, or --force to version the existing one.", orig_backup);
        }
    } else {
        _safe_cp(&src_path, &orig_backup);
    }
    
    // Perform the insertion
    run!("sed -i -e '\\#{}#r {}' -e '\\#{}#d' '{}'", marker_pattern, edit_file, marker_pattern, src_path);
    info!("Successfully inserted '{}' into '{}'.", new_func_name, src_path);
    
    0
}

fn do_done(args: Args) -> i32 {
    let func_name = args.get_or(1, "");
    
    if func_name.is_empty() {
        error!("Usage: func done <func_name>");
        return 1;
    }
    
    let orig_file = format!("./func/{}.orig.sh", func_name);
    let extracted_file = format!("./func/{}.extracted.sh", func_name);
    
    if test!(-f &orig_file) {
        let edit_name = _parse_meta_header("edit", &orig_file);
        let edit_file = if !edit_name.is_empty() {
            format!("./func/{}.edit.sh", edit_name)
        } else {
            String::new()
        };
        
        _safe_rm(&format!("-f '{}' '{}'", orig_file, edit_file));
        info!("Removed: {}", orig_file);
        if !edit_file.is_empty() {
            info!("Removed: {}", edit_file);
        }
    } else if test!(-f &extracted_file) {
        _safe_rm(&format!("-f '{}'", extracted_file));
        info!("Removed: {}", extracted_file);
    } else {
        fatal!("No files found for '{}'.", func_name);
    }
    
    0
}

fn do_clean(_args: Args) -> i32 {
    if get_var("opt_force") == "1" {
        if _force_prompt("Permanently delete ./func/ and all .orig backups?") {
            _safe_rm("-rf ./func");
            run!("find . -maxdepth 1 -name '*.orig*' -delete");
            info!("All artifacts removed.");
        } else {
            info!("Clean operation cancelled.");
        }
    } else {
        let has_backups = to_number!(run!("find . -maxdepth 1 -name '*.orig*' | wc -l")) > 0;
        if has_backups {
            mkdir_p("./orig");
            run!("find . -maxdepth 1 -name '*.orig*' -exec mv -t './orig/' {{}} +");
            info!("Archived backups to ./orig/");
        } else {
            info!("No backup files found to archive.");
        }
    }
    
    0
}

fn do_spy(args: Args) -> i32 {
    let func_name = args.get_or(1, "");
    let src_path = args.get_or(2, "");
    
    if func_name.is_empty() || src_path.is_empty() {
        error!("Usage: func spy <func> <src>");
        return 1;
    }
    
    let func_body = _extract_function_body(&func_name, &src_path);
    echo!("{}", func_body);
    
    0
}

fn do_extract(args: Args) -> i32 {
    let func_name = args.get_or(1, "");
    let src_path = args.get_or(2, "");
    
    if func_name.is_empty() || src_path.is_empty() {
        error!("Usage: func extract <func> <src>");
        return 1;
    }
    
    let func_body = _extract_function_body(&func_name, &src_path);
    if func_body.trim().is_empty() {
        fatal!("Function '{}' not found.", func_name);
    }
    
    mkdir_p("./func");
    let extract_file = format!("./func/{}.extracted.sh", func_name);
    write_file(&extract_file, &func_body);
    info!("Extracted function to '{}'", extract_file);
    
    0
}

fn do_check(args: Args) -> i32 {
    let orig_name = args.get_or(1, "");
    
    if orig_name.is_empty() {
        error!("Usage: func check <func_name>");
        return 1;
    }
    
    let orig_file = format!("./func/{}.orig.sh", orig_name);
    if !test!(-f &orig_file) {
        fatal!("Origin file not found: '{}'", orig_file);
    }
    
    let edit_name = _parse_meta_header("edit", &orig_file);
    let edit_file = format!("./func/{}.edit.sh", edit_name);
    if !test!(-f &edit_file) {
        fatal!("Corresponding edit file not found: '{}'", edit_file);
    }
    
    let cs1 = _checksum(&_extract_function_body(&orig_name, &orig_file));
    let cs2 = _checksum(&_extract_function_body(&edit_name, &edit_file));
    
    if cs1 == cs2 {
        info!("No changes detected.");
        1
    } else {
        info!("Changes detected.");
        0
    }
}

fn do_meta(args: Args) -> i32 {
    let file_name = args.get_or(1, "");
    
    if file_name.is_empty() {
        error!("Usage: func meta <func_file_in_func_dir>");
        return 1;
    }
    
    let file_path = format!("./func/{}", file_name);
    if !test!(-f &file_path) {
        fatal!("File not found in ./func/: '{}'", file_name);
    }
    
    let meta_line = run!("grep '^# FUNC_META' '{}'", file_path);
    echo!("{}", meta_line);
    
    0
}

fn do_flag(args: Args) -> i32 {
    let func_name = args.get_or(1, "");
    let new_name = args.get_or(2, "");
    let src_path = args.get_or(3, "");
    
    if func_name.is_empty() || new_name.is_empty() || src_path.is_empty() {
        error!("Usage: func flag <func> <new> <src>");
        return 1;
    }
    
    let line_num = _find_function_line(&func_name, &src_path);
    if line_num.trim().is_empty() {
        fatal!("Function '{}' not found.", func_name);
    }
    
    let marker_text = format!("# FUNC_INSERT ./func/{}.edit.sh", new_name);
    let marker_block = format!("\n\n\n\n{}", marker_text);
    
    run!("sed '{}i\\{}' '{}' > '{}.tmp' && mv '{}.tmp' '{}'", 
           line_num.trim(), marker_block, src_path, src_path, src_path, src_path);
    info!("Flag for '{}' inserted.", new_name);
    
    0
}

fn do_point(args: Args) -> i32 {
    let pattern = args.get_or(1, "");
    let src_path = args.get_or(2, "");
    
    if pattern.is_empty() || src_path.is_empty() {
        error!("Usage: func point <new> <src>");
        return 1;
    }
    
    let line_num = _find_flag_line(&pattern, &src_path);
    echo!("{}", line_num);
    
    0
}

fn do_where(args: Args) -> i32 {
    let func_name = args.get_or(1, "");
    let src_path = args.get_or(2, "");
    
    if func_name.is_empty() || src_path.is_empty() {
        error!("Usage: func where <func> <src> [--bash]");
        return 1;
    }
    
    let line_num = _find_function_line(&func_name, &src_path);
    if line_num.trim().is_empty() {
        echo!("-1");
    } else {
        echo!("{}", line_num.trim());
    }
    
    0
}

fn do_ls(args: Args) -> i32 {
    let src_path = args.get_or(1, "");
    
    if src_path.is_empty() {
        error!("Usage: func ls <src> [--bash]");
        return 1;
    }
    
    let functions = run!(r"grep -E '^[[:space:]]*[a-zA-Z0-9_]+\s*\(\s*\)\s*\{{' '{}' | sed -E 's/^[[:space:]]*//;s/\s*\(.*//'", src_path);
    echo!("{}", functions);
    
    0
}

fn do_find(args: Args) -> i32 {
    let pattern = args.get_or(1, "");
    let src_path = args.get_or(2, "");
    
    if pattern.is_empty() || src_path.is_empty() {
        error!("Usage: func find <pattern> <src> [--bash]");
        return 1;
    }
    
    let functions = run!(r"grep -E '^[[:space:]]*[a-zA-Z0-9_]+\s*\(\s*\)\s*\{{' '{}' | sed -E 's/^[[:space:]]*//;s/\s*\(.*//' | grep '{}'", src_path, pattern);
    echo!("{}", functions);
    
    0
}

// === HELPER FUNCTIONS ===

fn _yes_prompt(message: &str) -> bool {
    if get_var("opt_yes") == "1" {
        return true;
    }
    let hint = "(Use --yes to skip this prompt)";
    let full_prompt = format!("{} {}", message, hint);
    confirm!(&full_prompt)
}

fn _force_prompt(message: &str) -> bool {
    if get_var("opt_force") == "1" {
        return true;
    }
    let hint = "(Use --force to override)";
    let full_prompt = format!("{} {}", message, hint);
    confirm!(&full_prompt)
}
