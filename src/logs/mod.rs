use chrono::{DateTime, Local};
use colored_json::ToColoredJson;
use serde_json::Value;
use std::collections::HashMap;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

mod knex;
mod standard;

/// A compact log object that can be pretty printed
pub struct FormattedLog {
    pub date: DateTime<Local>,
    pub msg: String,
    pub extra: Option<HashMap<String, Value>>,
    pub color_overwrite: Option<Color>,
}

impl FormattedLog {
    /// Prints log to stdout
    /// It will properly apply default colors or overwrite with optional color parameter
    /// It also pretty prints json objects if possible
    pub fn print(&self) {
        let formatted_date = self.date.format("%Y-%m-%d %H:%M:%S:");

        print_color(&formatted_date.to_string(), Color::Cyan);
        print!(" ");
        print_color(
            &self.msg.trim(),
            self.color_overwrite.unwrap_or(Color::White),
        );

        if let Some(extra) = &self.extra {
            print!("\n");
            print_color(
                &serde_json::to_string_pretty(&extra)
                    .unwrap_or("".to_string())
                    .trim(),
                Color::Cyan,
            );
        }

        print!("\n");
    }
}

/// Describes a log that can be parsed (so a known log type) and can easily be printed
pub trait ParsableLog {
    fn format_compact(&self) -> FormattedLog;
    fn format_detailed(&self) -> FormattedLog;
    fn from_line(line: &str) -> Option<Self>
    where
        Self: Sized;
}

/// Internal function used to print known log types (like compact log and others)
/// It calls reset on stdout which should undo any coloring changes, but the lib
/// used for this doesn't seem to always work that way
fn print_color(text: &str, color: Color) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let mut to_write = text.clone().to_string();
    let mut color_to_write = Some(color);
    if let Ok(parsed) = serde_json::from_str::<Value>(text) {
        if parsed.is_object() || parsed.is_array() {
            if let Ok(colored) = to_write.to_colored_json_auto() {
                to_write = colored;
                // Ignore color to write in favor of colored_json_auto
                color_to_write = None;
            }
        }
    }

    if let Some(color_to_write) = color_to_write {
        stdout
            .set_color(ColorSpec::new().set_fg(Some(color_to_write)))
            .expect("Failed to set color");
    }
    if let Err(e) = write!(&mut stdout, "{}", to_write) {
        println!("Failed to write to stream {}", e);
    }
    if let Err(e) = stdout.reset() {
        println!("Failed to reset stream {}", e);
    }
}

pub fn try_parse_known_log(line: &str) -> Option<Box<dyn ParsableLog>> {
    if let Some(v) = standard::StandardLog::from_line(line) {
        Some(Box::new(v))
    } else if let Some(v) = knex::KnexLog::from_line(line) {
        Some(Box::new(v))
    } else {
        None
    }
}
