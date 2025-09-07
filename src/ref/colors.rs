// Shared Color System - Extracted from jynx architecture
// Complete 90+ semantic color palette for rich theme support
// Version: boxy v0.6.0+ (inherits jynx proven architecture)

pub const RESET: &str = "\x1B[0m";


/// Core color mapping function - supports 90+ semantic colors
pub fn get_color_code(color: &str) -> &'static str {
    match color {
        // === LEGACY COMPATIBILITY (v0.5.0 colors preserved) ===
        "red" => "\x1B[38;5;9m",
        "red2" => "\x1B[38;5;197m",
        "deep" => "\x1B[38;5;61m",
        "deep_green" => "\x1B[38;5;60m",
        "orange" => "\x1B[38;5;214m",
        "yellow" => "\x1B[33m",
        "green" => "\x1B[38;5;10m",
        "green2" => "\x1B[32m",
        "blue" => "\x1B[36m",
        "blue2" => "\x1B[38;5;39m",
        "cyan" => "\x1B[38;5;14m",
        "magenta" => "\x1B[35m",
        "purple" => "\x1B[38;5;213m",
        "purple2" => "\x1B[38;5;141m",
        "white" => "\x1B[38;5;247m",
        "white2" => "\x1B[38;5;15m",
        "grey" => "\x1B[38;5;242m",
        "grey2" => "\x1B[38;5;240m",
        "grey3" => "\x1B[38;5;237m",
        
        // === EXTENDED RED SPECTRUM ===
        "crimson" => "\x1B[38;5;196m",        // Pure red - critical alerts
        "ruby" => "\x1B[38;5;160m",           // Dark red - errors
        "coral" => "\x1B[38;5;203m",          // Red-orange - warnings
        "salmon" => "\x1B[38;5;209m",         // Light red-orange - notices
        "rose" => "\x1B[38;5;217m",           // Pink-red - highlights
        "brick" => "\x1B[38;5;124m",          // Dark brick red - severe
        
        // === EXTENDED ORANGE SPECTRUM ===
        "amber" => "\x1B[38;5;220m",          // Golden orange - attention
        "tangerine" => "\x1B[38;5;208m",      // Bright orange - active
        "peach" => "\x1B[38;5;216m",          // Light orange - soft alerts
        "rust" => "\x1B[38;5;166m",           // Dark orange - deprecation
        "bronze" => "\x1B[38;5;130m",         // Brown-orange - legacy
        "gold" => "\x1B[38;5;178m",           // Golden - achievements
        
        // === EXTENDED YELLOW SPECTRUM ===
        "lemon" => "\x1B[38;5;226m",          // Bright yellow - warnings
        "mustard" => "\x1B[38;5;184m",        // Muted yellow - caution
        "sand" => "\x1B[38;5;223m",           // Beige-yellow - neutral
        "cream" => "\x1B[38;5;230m",          // Light yellow - info
        "khaki" => "\x1B[38;5;143m",          // Olive-yellow - pending
        
        // === EXTENDED GREEN SPECTRUM ===
        "lime" => "\x1B[38;5;46m",            // Bright green - success
        "emerald" => "\x1B[38;5;34m",         // Pure green - completed
        "forest" => "\x1B[38;5;22m",          // Dark green - stable
        "mint" => "\x1B[38;5;121m",           // Light green - fresh
        "sage" => "\x1B[38;5;108m",           // Muted green - accepted
        "jade" => "\x1B[38;5;35m",            // Blue-green - verified
        "olive" => "\x1B[38;5;58m",           // Brown-green - archived
        
        // === EXTENDED BLUE SPECTRUM ===
        "azure" => "\x1B[38;5;33m",           // Sky blue - information
        "navy" => "\x1B[38;5;17m",            // Dark blue - system
        "royal" => "\x1B[38;5;21m",           // Royal blue - primary
        "ice" => "\x1B[38;5;159m",            // Light blue - secondary
        "steel" => "\x1B[38;5;67m",           // Grey-blue - infrastructure
        "teal" => "\x1B[38;5;30m",            // Blue-green - data
        "indigo" => "\x1B[38;5;54m",          // Deep blue - configuration
        
        // === EXTENDED PURPLE SPECTRUM ===
        "violet" => "\x1B[38;5;129m",         // Blue-purple - special
        "plum" => "\x1B[38;5;96m",            // Dark purple - reserved
        "lavender" => "\x1B[38;5;183m",       // Light purple - optional
        "orchid" => "\x1B[38;5;170m",         // Pink-purple - enhanced
        "mauve" => "\x1B[38;5;139m",          // Muted purple - metadata
        "amethyst" => "\x1B[38;5;98m",        // Deep purple - advanced
        
        // === EXTENDED CYAN SPECTRUM ===
        "aqua" => "\x1B[38;5;51m",            // Bright cyan - active data
        "turquoise" => "\x1B[38;5;45m",       // Blue-cyan - processing
        "sky" => "\x1B[38;5;117m",            // Light cyan - status
        "ocean" => "\x1B[38;5;31m",           // Deep cyan - persistence
        
        // === MONOCHROME SPECTRUM ===
        "black" => "\x1B[38;5;16m",           // Pure black - disabled
        "charcoal" => "\x1B[38;5;235m",       // Dark grey - inactive
        "slate" => "\x1B[38;5;244m",          // Medium grey - secondary
        "silver" => "\x1B[38;5;250m",         // Light grey - tertiary
        "pearl" => "\x1B[38;5;253m",          // Very light grey - background
        "snow" => "\x1B[38;5;255m",           // Pure white - emphasis
        
        // === SEMANTIC GROUPINGS ===
        
        // Error/Alert semantic colors
        "error" => "\x1B[38;5;196m",          // Critical error
        "warning" => "\x1B[38;5;220m",        // Warning state
        "danger" => "\x1B[38;5;160m",         // Dangerous operation
        "alert" => "\x1B[38;5;208m",          // Alert state
        
        // Success/Positive semantic colors
        "success" => "\x1B[38;5;46m",         // Success state
        "complete" => "\x1B[38;5;34m",        // Completion
        "verified" => "\x1B[38;5;35m",        // Verification
        "approved" => "\x1B[38;5;121m",       // Approval
        
        // Info/Neutral semantic colors
        "info" => "\x1B[38;5;33m",            // Information
        "note" => "\x1B[38;5;159m",           // Note/annotation
        "hint" => "\x1B[38;5;117m",           // Hint/tip
        "debug" => "\x1B[38;5;67m",           // Debug information
        
        // Process/State semantic colors
        "pending" => "\x1B[38;5;184m",        // Pending state
        "progress" => "\x1B[38;5;214m",       // In progress
        "blocked" => "\x1B[38;5;197m",        // Blocked state
        "queued" => "\x1B[38;5;143m",         // Queued state
        "active" => "\x1B[38;5;51m",          // Active state
        "inactive" => "\x1B[38;5;240m",       // Inactive state
        
        // Priority semantic colors
        "critical" => "\x1B[38;5;196m",       // Critical priority
        "high" => "\x1B[38;5;208m",           // High priority
        "medium" => "\x1B[38;5;220m",         // Medium priority
        "low" => "\x1B[38;5;250m",            // Low priority
        "trivial" => "\x1B[38;5;237m",        // Trivial priority
        
        // === ADVANCED VARIATIONS ===
        
        // Bright variants (high contrast)
        "bright_red" => "\x1B[38;5;9m",
        "bright_green" => "\x1B[38;5;10m",
        "bright_yellow" => "\x1B[38;5;11m",
        "bright_blue" => "\x1B[38;5;12m",
        "bright_magenta" => "\x1B[38;5;13m",
        "bright_cyan" => "\x1B[38;5;14m",
        
        // Dim variants (low contrast)
        "dim_red" => "\x1B[38;5;52m",
        "dim_green" => "\x1B[38;5;22m",
        "dim_yellow" => "\x1B[38;5;58m",
        "dim_blue" => "\x1B[38;5;17m",
        "dim_magenta" => "\x1B[38;5;54m",
        "dim_cyan" => "\x1B[38;5;23m",
        
        // Pastel variants (soft colors)
        "pastel_red" => "\x1B[38;5;217m",
        "pastel_green" => "\x1B[38;5;157m",
        "pastel_yellow" => "\x1B[38;5;230m",
        "pastel_blue" => "\x1B[38;5;159m",
        "pastel_purple" => "\x1B[38;5;183m",
        "pastel_orange" => "\x1B[38;5;223m",
        
        // Special control values
        "none" | "default" | "auto" => "",    // No color (use default)
        
        // Fallback for unknown colors - graceful degradation
        _ => "",
    }
}

/// Validate color name and provide fallback suggestions
pub fn validate_color(color: &str) -> Result<&'static str, String> {
    let color_code = get_color_code(color);
    if !color_code.is_empty() || color == "none" || color == "default" || color == "auto" {
        Ok(color_code)
    } else {
        // Provide fallback suggestions for common typos
        let suggestion = match color {
            c if c.contains("red") => Some("crimson"),
            c if c.contains("green") => Some("emerald"),
            c if c.contains("blue") => Some("azure"),
            c if c.contains("yellow") => Some("amber"),
            c if c.contains("purple") => Some("violet"),
            c if c.contains("orange") => Some("tangerine"),
            c if c.contains("grey") || c.contains("gray") => Some("slate"),
            _ => None,
        };
        
        if let Some(fallback) = suggestion {
            Err(format!("Unknown color '{}'. Did you mean '{}'?", color, fallback))
        } else {
            Err(format!("Unknown color '{}'. Use --help to see available colors.", color))
        }
    }
}


/// Get color categories for organized help display
pub fn get_color_categories() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("Legacy Colors (v0.5.0)", vec![
            "red", "red2", "deep", "deep_green", "orange", "yellow", "green", "green2",
            "blue", "blue2", "cyan", "magenta", "purple", "purple2", "white", "white2",
            "grey", "grey2", "grey3"
        ]),
        ("Red Spectrum", vec!["crimson", "ruby", "coral", "salmon", "rose", "brick"]),
        ("Orange Spectrum", vec!["amber", "tangerine", "peach", "rust", "bronze", "gold"]),
        ("Yellow Spectrum", vec!["lemon", "mustard", "sand", "cream", "khaki"]),
        ("Green Spectrum", vec!["lime", "emerald", "forest", "mint", "sage", "jade", "olive"]),
        ("Blue Spectrum", vec!["azure", "navy", "royal", "ice", "steel", "teal", "indigo"]),
        ("Purple Spectrum", vec!["violet", "plum", "lavender", "orchid", "mauve", "amethyst"]),
        ("Cyan Spectrum", vec!["aqua", "turquoise", "sky", "ocean"]),
        ("Monochrome", vec!["black", "charcoal", "slate", "silver", "pearl", "snow"]),
        ("Semantic Alerts", vec!["error", "warning", "danger", "alert"]),
        ("Semantic Success", vec!["success", "complete", "verified", "approved"]),
        ("Semantic Info", vec!["info", "note", "hint", "debug"]),
        ("Semantic States", vec!["pending", "progress", "blocked", "queued", "active", "inactive"]),
        ("Priority Levels", vec!["critical", "high", "medium", "low", "trivial"]),
    ]
}

/// Generate colored help text for CLI display
fn pad_cell(s: &str, width: usize) -> String {
    // Visible width approximation: count plain chars (ANSI not included here)
    let len = s.chars().count();
    if len >= width { return s.to_string(); }
    let pad = " ".repeat(width - len);
    format!("{}{}", s, pad)
}

pub fn generate_color_help() -> String {
    let mut help = String::new();
    help.push_str("COLORS:\n\n");

    let cols = 3usize;
    let cell_w = 22usize; // fits name nicely in most terminals

    for (category, colors) in get_color_categories() {
        help.push_str(&format!("{}:\n", category));

        let mut i = 0usize;
        while i < colors.len() {
            let mut row = String::new();
            for j in 0..cols {
                if i + j >= colors.len() { break; }
                let name = colors[i + j];
                let code = get_color_code(name);
                let cell = if !code.is_empty() {
                    format!("{}■ {}\x1B[0m", code, name)
                } else {
                    format!("  {}", name)
                };
                row.push_str(&pad_cell(&cell, cell_w));
            }
            row.push('\n');
            help.push_str("    ");
            help.push_str(&row);
            i += cols;
        }
        help.push('\n');
    }

    help
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legacy_colors_preserved() {
        // Ensure all v0.5.0 colors still work
        assert_eq!(get_color_code("red"), "\x1B[38;5;9m");
        assert_eq!(get_color_code("green"), "\x1B[38;5;10m");
        assert_eq!(get_color_code("blue2"), "\x1B[38;5;39m");
        assert_eq!(get_color_code("grey3"), "\x1B[38;5;237m");
    }

    #[test]
    fn test_extended_colors_available() {
        // Test new semantic colors
        assert_eq!(get_color_code("crimson"), "\x1B[38;5;196m");
        assert_eq!(get_color_code("emerald"), "\x1B[38;5;34m");
        assert_eq!(get_color_code("azure"), "\x1B[38;5;33m");
        assert_eq!(get_color_code("amber"), "\x1B[38;5;220m");
    }

    #[test]
    fn test_semantic_colors() {
        assert_eq!(get_color_code("error"), "\x1B[38;5;196m");
        assert_eq!(get_color_code("success"), "\x1B[38;5;46m");
        assert_eq!(get_color_code("warning"), "\x1B[38;5;220m");
        assert_eq!(get_color_code("info"), "\x1B[38;5;33m");
    }

    #[test]
    fn test_fallback_behavior() {
        // Unknown colors should return empty string
        assert_eq!(get_color_code("unknown"), "");
        assert_eq!(get_color_code("invalid_color"), "");
        
        // Control values
        assert_eq!(get_color_code("none"), "");
        assert_eq!(get_color_code("auto"), "");
    }

    #[test]
    fn test_color_validation() {
        // Valid colors
        assert!(validate_color("crimson").is_ok());
        assert!(validate_color("none").is_ok());
        
        // Invalid colors with suggestions
        let result = validate_color("redd");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("crimson"));
    }

    #[test]
    fn test_color_categories() {
        let categories = get_color_categories();
        assert!(!categories.is_empty());
        
        // Verify legacy colors are preserved
        let legacy_category = categories.iter().find(|(name, _)| name.contains("Legacy"));
        assert!(legacy_category.is_some());
    }
}
