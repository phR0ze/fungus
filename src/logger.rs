use chrono;
use colored::*;
use log;
use std::io;
use std::sync::Mutex;

use crate::prelude::*;

lazy_static! {
    // Arc isn't needed here as this is static
    static ref LOGOPTS: Mutex<LogOpts> = Mutex::new(LogOpts { level: log::Level::Info, color: true, buffer: false, silent: false, output: vec![] });
}

pub struct LogOpts {
    level: log::Level, // log level
    color: bool,       // use colored logging
    buffer: bool,      // use buffer for output?
    silent: bool,      // go silent when true
    output: Vec<u8>,   // buffer to use for output
}

pub struct Logger;
impl Logger {
    /// Initialize the global logger with the current Logger settings.
    pub fn init() -> Result<()> {
        let level = log::Level::Info;
        log::set_boxed_logger(Box::new(Logger {}))?;
        log::set_max_level(level.to_level_filter());
        Ok(())
    }

    /// Get the log `level` to use.
    pub fn level() -> log::Level {
        LOGOPTS.lock().unwrap().level
    }

    /// Set the log `level` to use.
    pub fn set_level(level: log::Level) {
        LOGOPTS.lock().unwrap().level = level;
        log::set_max_level(level.to_level_filter());
    }

    /// Check if logging should be in color.
    pub fn color() -> bool {
        LOGOPTS.lock().unwrap().color
    }

    /// Use color for logging if `yes` is true else no color.
    pub fn use_color(yes: bool) {
        LOGOPTS.lock().unwrap().color = yes;
    }

    /// Check if logging should go to buffer
    pub fn buffer() -> bool {
        LOGOPTS.lock().unwrap().buffer
    }

    /// Use buffer for logging if `yes` is true else io::stdout.
    pub fn use_buffer(yes: bool) {
        LOGOPTS.lock().unwrap().buffer = yes;
    }

    /// Check if in silent mode
    pub fn silent() -> bool {
        LOGOPTS.lock().unwrap().silent
    }

    /// Set silent for logging if `yes` is true else false
    pub fn be_silent(yes: bool) {
        LOGOPTS.lock().unwrap().silent = yes;
    }

    /// Return the data from the buffer as a String
    pub fn data() -> Result<String> {
        let mut opts = LOGOPTS.lock().unwrap();
        let result = match str::from_utf8(&opts.output) {
            Ok(x) => Ok(x.to_string()),
            Err(err) => Err(err.into()),
        };
        opts.output.clear();
        result
    }
}

impl log::Log for Logger {
    // Filter out incorrect levels
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= Logger::level()
    }

    // Log with correct level color and timestamp
    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let mut opts = LOGOPTS.lock().unwrap();
            if opts.silent {
                return;
            }

            // Get level prefix
            let level = if opts.color {
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
            if opts.buffer {
                writeln!(opts.output, "{:<5}[{}] {}", level, chrono::Local::now().format("%Y-%m-%d %H:%M:%S.%3f"), record.args()).unwrap();
            } else {
                writeln!(io::stdout(), "{:<5}[{}] {}", level, chrono::Local::now().format("%Y-%m-%d %H:%M:%S.%3f"), record.args()).unwrap();
            }
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
        Logger::use_buffer(true);

        Logger::use_color(false);
        log::error!("hello error");
        let data = Logger::data().unwrap();
        assert!(data.starts_with("ERROR["));
        Logger::use_color(true);
        log::error!("hello error");
        let data = Logger::data().unwrap();
        assert!(data.ends_with("hello error\n"));

        Logger::use_color(false);
        log::warn!("hello warn");
        let data = Logger::data().unwrap();
        assert!(data.starts_with("WARN ["));
        Logger::use_color(true);
        log::warn!("hello warn");
        let data = Logger::data().unwrap();
        assert!(data.ends_with("hello warn\n"));

        Logger::use_color(false);
        log::info!("hello info");
        let data = Logger::data().unwrap();
        assert!(data.starts_with("INFO ["));
        Logger::use_color(true);
        log::info!("hello info");
        let data = Logger::data().unwrap();
        assert!(data.ends_with("hello info\n"));

        // Test level
        Logger::use_color(false);
        log::debug!("hello debug");
        let data = Logger::data().unwrap();
        assert_eq!(data.len(), 0);
        Logger::set_level(log::Level::Debug);
        log::debug!("hello debug");
        let data = Logger::data().unwrap();
        assert!(data.starts_with("DEBUG["));
        Logger::use_color(true);
        log::debug!("hello debug");
        let data = Logger::data().unwrap();
        assert!(data.ends_with("hello debug\n"));

        // Test silent mode
        Logger::be_silent(true);
        log::info!("hello info");
        let data = Logger::data().unwrap();
        assert_eq!(data.len(), 0);

        // create log directory and file
        //assert_eq!(file1.mode().unwrap(), 0o100555);

        assert!(sys::remove_all(&tmpdir).is_ok());
    }
}
