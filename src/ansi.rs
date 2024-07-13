use crate::cli::ColorChoice;

/// Resets all colors and styles.
const RESET: &str = "\x1b[0m";

/// Bold.
const BOLD: &str = "\x1b[1m";

/// Dim.
const DIM: &str = "\x1b[2m";

/// Info (cyan).
const INFO: &str = "\x1b[36m";

/// Success (green).
const SUCCESS: &str = "\x1b[32m";

/// Error (magenta).
const ERROR: &str = "\x1b[35m";

pub struct Color {
    pub reset: &'static str,
    pub bold: &'static str,
    pub dim: &'static str,
    pub info: &'static str,
    pub success: &'static str,
    pub error: &'static str,
}

impl Color {
    pub fn from_color_choice(choice: ColorChoice) -> Self {
        match choice {
            ColorChoice::Always => Color {
                reset: RESET,
                bold: BOLD,
                dim: DIM,
                info: INFO,
                success: SUCCESS,
                error: ERROR,
            },
            ColorChoice::Never => Color {
                reset: "",
                bold: "",
                dim: "",
                info: "",
                success: "",
                error: "",
            },
        }
    }
}
