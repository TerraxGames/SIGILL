use core::fmt;

use colored::{ColoredString, Colorize};
use log::error;

use crate::constants;

static LOGGER: Logger = Logger;

pub fn init() -> Result<(), log::SetLoggerError> {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(constants::LOG_LEVEL))
}

pub fn hook_panic() {
    std::panic::set_hook(Box::new(|panic_info| {
        if let Some(payload) = panic_info.payload().downcast_ref::<String>() {
            error!(r#"{} has encountered a fatal error and cannot recover!
{}
{payload}
Please report this bug on our issue tracker: {}"#, constants::NAME, panic_info.location().unwrap_or(core::panic::Location::caller()), constants::ISSUE_TRACKER);
        } else {
            error!("{} has encountered a fatal error and cannot recover!\nPlease report this bug on our issue tracker: {}", constants::NAME, constants::ISSUE_TRACKER);
        }
    }));
}

// TODO: implement log files
pub struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= constants::LOG_LEVEL
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let origin = {
                if let Some(module_path) = record.module_path() {
                    format!("({}) ", module_path)
                } else {
                    String::new()
                }
            };
            println!("{origin}{}   {}", format_level(record.level()), colorize_args(record.level(), record.args()));
        }
    }

    fn flush(&self) {}
}

fn level_color(level: log::Level) -> colored::Color {
    match level {
        log::Level::Error => colored::Color::Red,
        log::Level::Warn => colored::Color::Yellow,
        log::Level::Info => colored::Color::Green,
        log::Level::Debug => colored::Color::Blue,
        log::Level::Trace => colored::Color::BrightBlack,
    }
}

fn format_level(level: log::Level) -> ColoredString {
    level.as_str().color(level_color(level))
}

fn colorize_args(level: log::Level, args: &fmt::Arguments) -> ColoredString {
    args.to_string().color(level_color(level))
}
