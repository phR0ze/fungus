use chrono;
use colored::*;
use log;
use std::io;

use crate::prelude::*;

/// Provides logging
#[derive(Debug)]
pub struct Logger {
    pub level: log::Level, // log level
    pub colored: bool,     // use colored logging
}
impl Logger {
    /// Create a new logger with defaults
    pub fn new() -> Logger {
        Logger { level: log::Level::Info, colored: true }
    }

    /// Use colored logging if `yes` is true else no color.
    pub fn set_colored(&mut self, yes: bool) -> &mut Self {
        self.colored = yes;
        self
    }

    /// Set the log `level` to use.
    pub fn set_level(&mut self, level: log::Level) -> &mut Self {
        self.level = level;
        self
    }

    /// Initialize the global logger with the current Logger settings.
    pub fn init(self) -> Result<()> {
        let level = self.level;
        log::set_boxed_logger(Box::new(self))?;
        log::set_max_level(level.to_level_filter());
        Ok(())
    }
}

impl log::Log for Logger {
    // Filter out incorrect levels
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.level
    }

    // Log with correct level color and timestamp
    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let level = if self.colored {
                match record.level() {
                    log::Level::Error => record.level().to_string().red(),
                    log::Level::Warn => record.level().to_string().yellow(),
                    log::Level::Info => record.level().to_string().cyan(),
                    log::Level::Debug => record.level().to_string().normal(),
                    log::Level::Trace => record.level().to_string().normal().dimmed(),
                }
            } else {
                record.level().to_string().normal()
            };

            // Write output to drains
            writeln!(io::stdout(), "{:<5}[{}] {}", level, chrono::Local::now().format("%Y-%m-%d %H:%M:%S.%3f"), record.args()).unwrap();
        }
    }

    fn flush(&self) {}
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    // Reusable teset setup
    struct Setup {
        temp: PathBuf,
    }
    impl Setup {
        fn init() -> Self {
            let setup = Self { temp: PathBuf::from("tests/temp").abs().unwrap() };
            sys::mkdir(&setup.temp).unwrap();
            setup
        }
    }

    #[test]
    fn test_init() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("logger_log");
        let file1 = tmpdir.mash("file1");
        assert!(sys::remove_all(&tmpdir).is_ok());

        // Log output
        let mut logger = Logger::new().init().unwrap();
        log::error!("hello error");
        // log::warn!("hello warn");
        // log::info!("hello info");
        // log::debug!("hello debug");
        // log::trace!("hello trace");

        // create log directory and file
        //assert_eq!(file1.mode().unwrap(), 0o100555);

        assert!(sys::remove_all(&tmpdir).is_ok());
    }
}
