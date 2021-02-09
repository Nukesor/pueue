use crossterm::style::Color;
use pueue_lib::settings::Settings;

/// Color wrapper for actual colors depending on settings
/// Using dark colors if dark_mode is enabled
pub struct Colors {
    /// red color
    red: Color,
    /// green color
    green: Color,
    /// white color
    white: Color,
    /// yellow color
    yellow: Color,
}

impl Colors {
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
}
