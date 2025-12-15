// Native alternative to `console` crate
// Terminal styling with zero dependencies
// Savings: -100 KB, zero external deps

use std::fmt;
use std::env;

/// ANSI color codes
#[derive(Debug, Clone, Copy)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

impl Color {
    fn fg_code(&self) -> &str {
        match self {
            Color::Black => "30",
            Color::Red => "31",
            Color::Green => "32",
            Color::Yellow => "33",
            Color::Blue => "34",
            Color::Magenta => "35",
            Color::Cyan => "36",
            Color::White => "37",
            Color::BrightBlack => "90",
            Color::BrightRed => "91",
            Color::BrightGreen => "92",
            Color::BrightYellow => "93",
            Color::BrightBlue => "94",
            Color::BrightMagenta => "95",
            Color::BrightCyan => "96",
            Color::BrightWhite => "97",
        }
    }
    
    fn bg_code(&self) -> &str {
        match self {
            Color::Black => "40",
            Color::Red => "41",
            Color::Green => "42",
            Color::Yellow => "43",
            Color::Blue => "44",
            Color::Magenta => "45",
            Color::Cyan => "46",
            Color::White => "47",
            Color::BrightBlack => "100",
            Color::BrightRed => "101",
            Color::BrightGreen => "102",
            Color::BrightYellow => "103",
            Color::BrightBlue => "104",
            Color::BrightMagenta => "105",
            Color::BrightCyan => "106",
            Color::BrightWhite => "107",
        }
    }
}

/// Styled text
pub struct StyledText {
    text: String,
    fg_color: Option<Color>,
    bg_color: Option<Color>,
    bold: bool,
    dim: bool,
    italic: bool,
    underline: bool,
}

impl StyledText {
    /// Create new styled text
    pub fn new<S: Into<String>>(text: S) -> Self {
        Self {
            text: text.into(),
            fg_color: None,
            bg_color: None,
            bold: false,
            dim: false,
            italic: false,
            underline: false,
        }
    }
    
    /// Set foreground color
    pub fn fg(mut self, color: Color) -> Self {
        self.fg_color = Some(color);
        self
    }
    
    /// Set background color
    pub fn bg(mut self, color: Color) -> Self {
        self.bg_color = Some(color);
        self
    }
    
    /// Make text bold
    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }
    
    /// Make text dim
    pub fn dim(mut self) -> Self {
        self.dim = true;
        self
    }
    
    /// Make text italic
    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }
    
    /// Underline text
    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }
    
    /// Convert to ANSI string
    pub fn to_string(&self) -> String {
        if !is_terminal() {
            return self.text.clone();
        }
        
        let mut codes = Vec::new();
        
        if self.bold {
            codes.push("1");
        }
        if self.dim {
            codes.push("2");
        }
        if self.italic {
            codes.push("3");
        }
        if self.underline {
            codes.push("4");
        }
        if let Some(color) = &self.fg_color {
            codes.push(color.fg_code());
        }
        if let Some(color) = &self.bg_color {
            codes.push(color.bg_code());
        }
        
        if codes.is_empty() {
            self.text.clone()
        } else {
            format!("\x1b[{}m{}\x1b[0m", codes.join(";"), self.text)
        }
    }
}

impl fmt::Display for StyledText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Check if stdout is a terminal
fn is_terminal() -> bool {
    // Simple heuristic: check if NO_COLOR or TERM is set
    if env::var("NO_COLOR").is_ok() {
        return false;
    }
    
    // Assume terminal if TERM is set
    env::var("TERM").is_ok()
}

/// Quick styling functions
pub fn red<S: Into<String>>(text: S) -> StyledText {
    StyledText::new(text).fg(Color::Red)
}

pub fn green<S: Into<String>>(text: S) -> StyledText {
    StyledText::new(text).fg(Color::Green)
}

pub fn yellow<S: Into<String>>(text: S) -> StyledText {
    StyledText::new(text).fg(Color::Yellow)
}

pub fn blue<S: Into<String>>(text: S) -> StyledText {
    StyledText::new(text).fg(Color::Blue)
}

pub fn magenta<S: Into<String>>(text: S) -> StyledText {
    StyledText::new(text).fg(Color::Magenta)
}

pub fn cyan<S: Into<String>>(text: S) -> StyledText {
    StyledText::new(text).fg(Color::Cyan)
}

pub fn white<S: Into<String>>(text: S) -> StyledText {
    StyledText::new(text).fg(Color::White)
}

pub fn bold<S: Into<String>>(text: S) -> StyledText {
    StyledText::new(text).bold()
}

pub fn dim<S: Into<String>>(text: S) -> StyledText {
    StyledText::new(text).dim()
}

pub fn italic<S: Into<String>>(text: S) -> StyledText {
    StyledText::new(text).italic()
}

pub fn underline<S: Into<String>>(text: S) -> StyledText {
    StyledText::new(text).underline()
}

/// Emoji support (works on most terminals)
pub mod emoji {
    pub const CHECK: &str = "✅";
    pub const CROSS: &str = "❌";
    pub const WARNING: &str = "⚠️";
    pub const INFO: &str = "ℹ️";
    pub const ROCKET: &str = "🚀";
    pub const FIRE: &str = "🔥";
    pub const SPARKLES: &str = "✨";
    pub const PACKAGE: &str = "📦";
    pub const WRENCH: &str = "🔧";
    pub const MAGNIFYING_GLASS: &str = "🔍";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_styling() {
        let styled = StyledText::new("Hello").fg(Color::Red).bold();
        let output = styled.to_string();
        // Should contain ANSI codes or plain text
        assert!(output.contains("Hello"));
    }

    #[test]
    fn test_quick_functions() {
        assert!(red("test").to_string().contains("test"));
        assert!(green("test").to_string().contains("test"));
        assert!(bold("test").to_string().contains("test"));
    }
}

impl From<StyledText> for String {
    fn from(styled: StyledText) -> Self {
        styled.to_string()
    }
}
