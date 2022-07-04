use std::io::stdout;

use pueue_lib::settings::Settings;

use comfy_table::Cell;
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
            }
        } else {
            Self {
                green: Color::Green,
                red: Color::Red,
                yellow: Color::Yellow,
            }
        }
    }

    fn map_color(&self, color: Color) -> Color {
        match color {
            Color::Green => self.green,
            Color::Red => self.red,
            Color::Yellow => self.yellow,
            _ => color,
        }
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
            styled = styled.with(self.map_color(color));
        }
        if let Some(attribute) = attribute {
            styled = styled.attribute(attribute);
        }

        styled.to_string()
    }

    /// A helper method to produce styled Comfy-table cells.
    /// Use this anywhere you need to create Comfy-table cells, so that the correct
    /// colors are used depending on the current color mode and dark-mode preset.
    pub fn styled_cell<T: ToString>(
        &self,
        text: T,
        color: Option<Color>,
        attribute: Option<Attribute>,
    ) -> Cell {
        let mut cell = Cell::new(text.to_string());
        // Styling disabled
        if !self.enabled {
            return cell;
        }

        if let Some(color) = color {
            cell = cell.fg(self.map_color(color));
        }
        if let Some(attribute) = attribute {
            cell = cell.add_attribute(attribute);
        }

        cell
    }
}
