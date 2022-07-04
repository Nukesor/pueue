use std::io::stdout;

use pueue_lib::settings::Settings;

use crossterm::style::{style, Attribute, Color, Stylize};
use crossterm::tty::IsTty;
/// OutputStyle wrapper for actual colors depending on settings
/// - Enables styles if color mode is 'always', or if color mode is 'auto' and output is a tty.
/// - Using dark colors if dark_mode is enabled
pub struct OutputStyle {
    /// red color
    red: Color,
    /// green color
    green: Color,
    /// white color
    white: Color,
    /// yellow color
    yellow: Color,
}

impl OutputStyle {
    /// init color-scheme depending on settings
    pub const fn new(settings: &Settings) -> Self {
        if settings.client.dark_mode {
            Self {
                green: Color::DarkGreen,
                red: Color::DarkRed,
                yellow: Color::DarkYellow,
                white: Color::White,
            }
        } else {
            Self {
                green: Color::Green,
                red: Color::Red,
                yellow: Color::Yellow,
                white: Color::White,
            }
        }
    }

    /// return green color
    pub const fn green(&self) -> Color {
        self.green
    }
    /// return red color
    pub const fn red(&self) -> Color {
        self.red
    }
    /// return yellow color
    pub const fn yellow(&self) -> Color {
        self.yellow
    }
    /// return white color
    pub const fn white(&self) -> Color {
        self.white
    }

    /// This is a helper method with the purpose of easily styling text,
    /// while also prevent styling if we're printing to a non-tty output.
    /// If there's any kind of styling in the code, it should be done with the help of this method.
    pub fn style_text<T: ToString>(
        &self,
        text: T,
        color: Option<Color>,
        attribute: Option<Attribute>,
    ) -> String {
        let text = text.to_string();
        // No tty, we aren't allowed to do any styling
        if !stdout().is_tty() {
            return text;
        }

        let mut styled = style(text);
        if let Some(color) = color {
            styled = styled.with(color);
        }
        if let Some(attribute) = attribute {
            styled = styled.attribute(attribute);
        }

        styled.to_string()
    }
}
