


/// Validate style input
pub fn validate_box_style(style: &str) -> Result<(), String> {
    let valid_styles = vec!["normal", "rounded", "double", "heavy", "ascii"];
    if !valid_styles.contains(&style) {
        return Err(format!("Invalid style '{}'. Valid styles: {}", style, valid_styles.join(", ")));
    }
    Ok(())
}

//WARN: this must contain all box characters!
pub const BOX_CHARS:&str = "┌┐└┘─│├┤┼╭╮╰╯═║╠╣╬╔╗╚╝━┃┣┫╋┏┓┗┛+-|";

pub struct BoxStyle {
    pub top_left: &'static str,
    pub top_right: &'static str,
    pub bottom_left: &'static str,
    pub bottom_right: &'static str,
    pub horizontal: &'static str,
    pub vertical: &'static str,
    pub tee_left: &'static str,
    pub tee_right: &'static str,
    #[allow(dead_code)]
    pub cross: &'static str,
}

pub const NORMAL: BoxStyle = BoxStyle {
    top_left: "┌", top_right: "┐",
    bottom_left: "└", bottom_right: "┘",
    horizontal: "─", vertical: "│",
    tee_left: "├", tee_right: "┤", cross: "┼",
};

pub const ROUNDED: BoxStyle = BoxStyle {
    top_left: "╭", top_right: "╮",
    bottom_left: "╰", bottom_right: "╯",
    horizontal: "─", vertical: "│",
    tee_left: "├", tee_right: "┤", cross: "┼",
};

pub const DOUBLE: BoxStyle = BoxStyle {
    top_left: "╔", top_right: "╗",
    bottom_left: "╚", bottom_right: "╝",
    horizontal: "═", vertical: "║",
    tee_left: "╠", tee_right: "╣", cross: "╬",
};

pub const HEAVY: BoxStyle = BoxStyle {
    top_left: "┏", top_right: "┓",
    bottom_left: "┗", bottom_right: "┛",
    horizontal: "━", vertical: "┃",
    tee_left: "┣", tee_right: "┫", cross: "╋",
};

pub const ASCII: BoxStyle = BoxStyle {
    top_left: "+", top_right: "+",
    bottom_left: "+", bottom_right: "+",
    horizontal: "-", vertical: "|",
    tee_left: "+", tee_right: "+", cross: "+",
};

