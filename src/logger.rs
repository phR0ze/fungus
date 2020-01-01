use chrono;
use colored::*;
use log;
use std::io;
use std::sync::{Arc, Mutex};

use crate::prelude::*;

lazy_static! {
    static ref BUFFER: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(vec![]));
}

/// Provides logging
#[derive(Debug)]
pub struct Logger {
    pub level: log::Level, // log level
    pub colored: bool,     // use colored logging
}
impl Logger {
    /// Initialize the global logger with the current Logger settings.
    pub fn init() -> Result<()> {
        let level = log::Level::Info;
        let logger = Logger { level: level, colored: true };
        log::set_boxed_logger(Box::new(logger))?;
        log::set_max_level(level.to_level_filter());
        Ok(())
    }

    /// Return the data from the buffer as a String
    pub fn buffer() -> Result<String> {
        let result = match str::from_utf8(&BUFFER.lock().unwrap()[..]) {
            Ok(x) => Ok(x.to_string()),
            Err(err) => Err(err.into()),
        };
        BUFFER.lock().unwrap().clear();
        result
    }

    /// Set the log `level` to use.
    pub fn set_level(&mut self, level: log::Level) -> &mut Self {
        self.level = level;
        log::set_max_level(level.to_level_filter());
        self
    }

    /// Use colored logging if `yes` is true else no color.
    pub fn set_colored(&mut self, yes: bool) -> &mut Self {
        self.colored = yes;
        self
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
            writeln!(BUFFER.lock().unwrap(), "{:<5}[{}] {}", level, chrono::Local::now().format("%Y-%m-%d %H:%M:%S.%3f"), record.args()).unwrap();
            //writeln!(io::stdout(), "{:<5}[{}] {}", level, chrono::Local::now().format("%Y-%m-%d %H:%M:%S.%3f"), record.args()).unwrap();
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
        Logger::init().unwrap();
        log::error!("hello error");
        assert!(Logger::buffer().unwrap().ends_with("hello error\n"));
        log::warn!("hello warn");
        assert!(Logger::buffer().unwrap().ends_with("hello warn\n"));
        log::warn!("hello info");
        assert!(Logger::buffer().unwrap().ends_with("hello info\n"));

        // create log directory and file
        //assert_eq!(file1.mode().unwrap(), 0o100555);

        assert!(sys::remove_all(&tmpdir).is_ok());
    }
}
