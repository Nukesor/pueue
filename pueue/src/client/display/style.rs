use comfy_table::{Attribute as ComfyAttribute, Cell, Color as ComfyColor};
use crossterm::style::{style, Attribute, Color, Stylize};
use pueue_lib::settings::Settings;

/// OutputStyle wrapper for actual colors depending on settings
/// - Enables styles if color mode is 'always', or if color mode is 'auto' and output is a tty.
/// - Using dark colors if dark_mode is enabled
#[derive(Debug, Clone)]
pub struct OutputStyle {
    /// Whether or not ANSI styling is enabled
    pub enabled: bool,
    /// Whether dark mode is enabled.
    pub dark_mode: bool,
}

impl OutputStyle {
    /// init color-scheme depending on settings
    pub const fn new(settings: &Settings, enabled: bool) -> Self {
        Self {
            enabled,
            dark_mode: settings.client.dark_mode,
        }
    }

    /// Return the desired crossterm color depending on whether we're in dark mode or not.
    fn map_color(&self, color: Color) -> Color {
        if self.dark_mode {
            match color {
                Color::Green => Color::DarkGreen,
                Color::Red => Color::DarkRed,
                Color::Yellow => Color::DarkYellow,
                _ => color,
            }
        } else {
            color
        }
    }

    /// Return the desired comfy_table color depending on whether we're in dark mode or not.
    fn map_comfy_color(&self, color: Color) -> ComfyColor {
        if self.dark_mode {
            return match color {
                Color::Green => ComfyColor::DarkGreen,
                Color::Red => ComfyColor::DarkRed,
                Color::Yellow => ComfyColor::DarkYellow,
                _ => ComfyColor::White,
            };
        }

        match color {
            Color::Green => ComfyColor::Green,
            Color::Red => ComfyColor::Red,
            Color::Yellow => ComfyColor::Yellow,
            _ => ComfyColor::White,
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
        // Styling disabled
        if !self.enabled {
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
        attribute: Option<ComfyAttribute>,
    ) -> Cell {
        let mut cell = Cell::new(text.to_string());
        // Styling disabled
        if !self.enabled {
            return cell;
        }

        if let Some(color) = color {
            cell = cell.fg(self.map_comfy_color(color));
        }
        if let Some(attribute) = attribute {
            cell = cell.add_attribute(attribute);
        }

        cell
    }
}
