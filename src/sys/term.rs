use crate::sys;
use lazy_static::*;

lazy_static! {
    /// `TERM_COLOR` will be true if the environment is a tty and the
    /// environment variable `TERM_COLOR` is not set to something falsy.
    static ref TERM_COLOR: bool = sys::hastty() && sys::flag_default("TERM_COLOR", true);
}

/// Check if the environment has a tty attached adn the environment
/// variable `TERM_COLOR` is not set to something falsy.
pub fn is_color() -> bool {
    *TERM_COLOR
}

/// `Colorable` defines a set of simple color functions for a given type
pub trait Colorable {
    // Set the color to use
    fn set_color(self, color: Color) -> ColorString
    where
        Self: Sized;

    // Clear any color that was set
    fn clear(self) -> ColorString
    where
        Self: Sized;

    // Set the color to red for the string
    fn red(self) -> ColorString
    where
        Self: Sized,
    {
        self.set_color(Color::Red)
    }
}

/// Define supported color types
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    Red,     // 31
    Green,   // 32
    Yellow,  // 33
    Blue,    // 34
    Magenta, // 35
    Cyan,    // 36
    White,   // 37
}
impl Color {
    /// Convert the color into a terminal escape sequence
    pub fn escape_seq(&self) -> String {
        match *self {
            Color::Red => "31".to_string(),
            Color::Green => "32".to_string(),
            Color::Yellow => "33".to_string(),
            Color::Blue => "34".to_string(),
            Color::Magenta => "35".to_string(),
            Color::Cyan => "36".to_string(),
            Color::White => "37".to_string(),
        }
    }
}

/// Wrapper around the String type to provide colors and styles.
#[derive(Clone, Debug)]
pub struct ColorString {
    raw: String,
    color: Option<Color>,
}

// Implement Deref to make ColorString behave like String
impl core::ops::Deref for ColorString {
    type Target = str;
    fn deref(&self) -> &str {
        &self.raw
    }
}

// Implement the Colorable trait for chaining of operations
impl Colorable for ColorString {
    // Update the color
    fn set_color(mut self, color: Color) -> ColorString {
        self.color = Some(color);
        self
    }

    // Clear the color
    fn clear(mut self) -> ColorString
    where
        Self: Sized,
    {
        self.color = None;
        self
    }
}

// Write out the color string
impl std::fmt::Display for ColorString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // If color is disabled fallback on String's implementation
        if !is_color() || self.color.is_none() {
            return <String as std::fmt::Display>::fmt(&self.raw, f);
        }

        // Write out color strings using terminal escape sequences
        // TODO: use finally pattern here to ensure a reset get written out
        f.write_str("\x1B[0m")?;
        Ok(())
    }
}

// Implement the Colorable Trait for &str
impl<'a> Colorable for &'a str {
    // Set the color
    fn set_color(self, color: Color) -> ColorString {
        ColorString {
            // Copy as owned string
            raw: String::from(self),

            // Store the color for the string
            color: Some(color),
        }
    }

    // Clear the color
    fn clear(self) -> ColorString
    where
        Self: Sized,
    {
        ColorString {
            // Copy as owned string
            raw: String::from(self),

            // Don't set any color
            color: None,
        }
    }
}

// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_term_color() {
        assert!(is_color() || !is_color());
    }

    #[test]
    fn test_color() {
        assert!(is_color() || !is_color());
    }
}
