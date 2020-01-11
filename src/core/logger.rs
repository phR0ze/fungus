use chrono;
use colored::*;
use log;
use std::io;
use std::sync::Mutex;

use crate::prelude::*;

lazy_static! {
    // Arc isn't needed for reference counting as this is static
    static ref LOGOPTS: Mutex<LogOpts> = Mutex::new(LogOpts {
        level: log::Level::Info,
        init: false,   // already initialized?
        silent: false, // allow output by default
        color: true,   // use color by default
        file: None,    // don't log to file by default
        stdout: true,  // log to stdout by defaultr
        buffer: false, // don't log to buffer by default
        output: vec![] // set buffer to use
    });
}

struct LogOpts {
    level: log::Level,  // log level
    init: bool,         // initialized?
    silent: bool,       // be silent?
    color: bool,        // use color in output?
    file: Option<File>, // use file for output?
    stdout: bool,       // use stdout for output?
    buffer: bool,       // use buffer for output?
    output: Vec<u8>,    // buffer to use for output
}

pub struct Logger;
impl Logger {
    /// Initialize the global logger with the current Logger settings.
    pub fn init() -> Result<()> {
        let mut opts = LOGOPTS.lock().unwrap();
        if !opts.init {
            opts.init = true;
            let level = log::Level::Info;
            log::set_boxed_logger(Box::new(Logger {}))?;
            log::set_max_level(level.to_level_filter());
        }
        Ok(())
    }

    /// Get the log `level` to use.
    pub fn level() -> log::Level {
        LOGOPTS.lock().unwrap().level
    }

    /// Convert a string to a log level
    pub fn level_from_str<T: AsRef<str>>(level: T) -> log::Level {
        match level.as_ref().to_lowercase().as_ref() {
            "error" => log::Level::Error,
            "warn" => log::Level::Warn,
            "debug" => log::Level::Debug,
            "trace" => log::Level::Trace,
            _ => log::Level::Info,
        }
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

    /// Enable color for logging. Only affects stdout, buffer and file logging have color
    /// always disabled.
    pub fn enable_color() {
        LOGOPTS.lock().unwrap().color = true;
    }

    /// Disable color for logging.
    pub fn disable_color() {
        LOGOPTS.lock().unwrap().color = false;
    }

    /// Check if logging should go to file
    pub fn file() -> bool {
        LOGOPTS.lock().unwrap().file.is_some()
    }

    /// Enable file logging to `path`
    pub fn enable_file<T: AsRef<Path>>(path: T) -> Result<()> {
        let mut opts = LOGOPTS.lock().unwrap();

        // Close existing file if it exists
        if opts.file.is_some() {
            opts.file.as_ref().unwrap().sync_all()?;
            opts.file = None;
        }

        // Ensure the log diretory exists
        sys::mkdir(path.as_ref().dir()?)?;

        // Open the log file for appending
        opts.file = Some(OpenOptions::new().create(true).append(true).open(path)?);
        Ok(())
    }

    /// Disable current file logging by closing the file, flushing the content.
    pub fn disable_file() -> Result<()> {
        let mut opts = LOGOPTS.lock().unwrap();
        if opts.file.is_some() {
            opts.file.as_ref().unwrap().sync_all()?;
            opts.file = None;
        }
        Ok(())
    }

    /// Check if logging should go to buffer
    pub fn buffer() -> bool {
        LOGOPTS.lock().unwrap().buffer
    }

    /// Enable buffer logging which will disable stdout logging
    pub fn enable_buffer() {
        let mut opts = LOGOPTS.lock().unwrap();
        opts.buffer = true;
        opts.stdout = false;
    }

    /// Disable buffer logging which enables stdout logging
    pub fn disable_buffer() {
        let mut opts = LOGOPTS.lock().unwrap();
        opts.buffer = false;
        opts.stdout = true;
    }

    /// Flush buffer to stdout if not in silent mode else just drop the buffer contents
    pub fn flush_buffer() {
        let mut opts = LOGOPTS.lock().unwrap();
        if opts.output.len() != 0 {
            if !opts.silent {
                if let Ok(x) = str::from_utf8(&opts.output) {
                    let data = x.to_string();
                    write!(io::stdout(), "{}", data).unwrap();
                }
            }
            opts.output.clear();
        }
    }

    /// Check if in silent mode
    pub fn silent() -> bool {
        LOGOPTS.lock().unwrap().silent
    }

    /// Enable silence i.e. no logging to any drain. When silence is disabled the original drains
    /// will be used.
    pub fn enable_silence() {
        LOGOPTS.lock().unwrap().silent = true;
    }

    /// Disable silence allowing original drains to function again.
    pub fn disable_silence() {
        LOGOPTS.lock().unwrap().silent = false;
    }

    /// Check if logging should go to stdout
    pub fn stdout() -> bool {
        LOGOPTS.lock().unwrap().stdout
    }

    /// Enable stdout logging
    pub fn enable_stdout() {
        LOGOPTS.lock().unwrap().stdout = true;
    }

    /// Disable stdout logging
    pub fn disable_stdout() {
        LOGOPTS.lock().unwrap().stdout = false;
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

    /// Log fatal error message than exit the process
    pub fn fatal(args: fmt::Arguments, level: log::Level, &(target, module_path, file, line): &(&str, &'static str, &'static str, u32)) {
        let mut opts = LOGOPTS.lock().unwrap();
        if opts.silent {
            return;
        }
        let record = log::Record::builder()
            .args(args)
            .level(level)
            .target(target)
            .module_path_static(Some(module_path))
            .file_static(Some(file))
            .line(Some(line))
            .build();

        // Get level prefix
        let level = "FATAL".to_string();
        let level_color = if opts.color { level.red() } else { level.normal() };

        // Write output to drains
        if opts.stdout {
            writeln!(io::stdout(), "{:<5}[{}] {}", level_color, chrono::Local::now().format("%Y-%m-%d %H:%M:%S.%3f"), record.args()).unwrap();
        }
        if opts.buffer {
            writeln!(opts.output, "{:<5}[{}] {}", level_color, chrono::Local::now().format("%Y-%m-%d %H:%M:%S.%3f"), record.args()).unwrap();
        }
        if opts.file.is_some() {
            let mut file = opts.file.as_ref().unwrap();
            writeln!(file, "{:<5}[{}] {}", level, chrono::Local::now().format("%Y-%m-%d %H:%M:%S.%3f"), record.args()).unwrap();
        }
        std::process::exit(1);
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
            let level = record.level().to_string();
            let level_color = if opts.color {
                match record.level() {
                    log::Level::Error => level.red(),
                    log::Level::Warn => level.yellow(),
                    log::Level::Info => level.cyan(),
                    log::Level::Debug => level.normal(),
                    log::Level::Trace => level.normal().dimmed(),
                }
            } else {
                level.normal()
            };

            // Write output to drains
            if opts.stdout {
                writeln!(io::stdout(), "{:<5}[{}] {}", level_color, chrono::Local::now().format("%Y-%m-%d %H:%M:%S.%3f"), record.args()).unwrap();
            }
            if opts.buffer {
                writeln!(opts.output, "{:<5}[{}] {}", level_color, chrono::Local::now().format("%Y-%m-%d %H:%M:%S.%3f"), record.args()).unwrap();
            }
            if opts.file.is_some() {
                let mut file = opts.file.as_ref().unwrap();
                writeln!(file, "{:<5}[{}] {}", level, chrono::Local::now().format("%Y-%m-%d %H:%M:%S.%3f"), record.args()).unwrap();
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
    fn test_level_from_str() {
        assert_eq!(Logger::level_from_str("error"), log::Level::Error);
        assert_eq!(Logger::level_from_str("Error"), log::Level::Error);
        assert_eq!(Logger::level_from_str("warn"), log::Level::Warn);
        assert_eq!(Logger::level_from_str("Warn"), log::Level::Warn);
        assert_eq!(Logger::level_from_str("info"), log::Level::Info);
        assert_eq!(Logger::level_from_str("Info"), log::Level::Info);
        assert_eq!(Logger::level_from_str("debug"), log::Level::Debug);
        assert_eq!(Logger::level_from_str("Debug"), log::Level::Debug);
        assert_eq!(Logger::level_from_str("trace"), log::Level::Trace);
        assert_eq!(Logger::level_from_str("Trace"), log::Level::Trace);
    }

    #[test]
    fn test_init() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("logger_log");
        let file1 = tmpdir.mash("file1");
        assert!(sys::remove_all(&tmpdir).is_ok());

        // Init
        Logger::init().unwrap();
        if Logger::stdout() {
            Logger::disable_stdout();
        }
        if !Logger::buffer() {
            Logger::enable_buffer();
        }
        if Logger::color() {
            Logger::disable_color();
        }

        // Test log levels
        error!("hello error");
        let data = Logger::data().unwrap();
        assert!(data.starts_with("ERROR["));
        Logger::enable_color();
        error!("hello error");
        let data = Logger::data().unwrap();
        assert!(data.ends_with("hello error\n"));

        // can't test directly
        // fatal!("hello fatal");
        // let data = Logger::data().unwrap();
        // assert!(data.starts_with("FATAL["));
        // assert!(data.ends_with("hello fatal\n"));

        Logger::disable_color();
        warn!("hello warn");
        let data = Logger::data().unwrap();
        assert!(data.starts_with("WARN ["));
        Logger::enable_color();
        warn!("hello warn");
        let data = Logger::data().unwrap();
        assert!(data.ends_with("hello warn\n"));

        Logger::disable_color();
        info!("hello info");
        let data = Logger::data().unwrap();
        assert!(data.starts_with("INFO ["));
        Logger::enable_color();
        info!("hello info");
        let data = Logger::data().unwrap();
        assert!(data.ends_with("hello info\n"));

        // Test level
        Logger::disable_color();
        debug!("hello debug");
        let data = Logger::data().unwrap();
        assert_eq!(data.len(), 0);
        Logger::set_level(log::Level::Trace);
        debug!("hello debug");
        let data = Logger::data().unwrap();
        assert!(data.starts_with("DEBUG["));
        Logger::enable_color();
        debug!("hello debug");
        let data = Logger::data().unwrap();
        assert!(data.ends_with("hello debug\n"));

        Logger::disable_color();
        trace!("hello trace");
        let data = Logger::data().unwrap();
        assert!(data.starts_with("TRACE["));
        Logger::enable_color();
        debug!("hello trace");
        assert!(data.ends_with("hello trace\n"));

        // Test silent mode
        Logger::flush_buffer();
        Logger::disable_color();
        if !Logger::silent() {
            Logger::enable_silence();
        }
        info!("hello info");
        assert_eq!(Logger::data().unwrap().len(), 0);
        Logger::flush_buffer();
        Logger::disable_silence();
        Logger::disable_buffer();

        // Test stdio
        info!("hello info");
        Logger::enable_stdout();
        trace!("hello trace");
        Logger::disable_stdout();

        // Test file logging
        assert_eq!(file1.exists(), false);
        assert!(Logger::enable_file(&file1).is_ok());
        assert!(Logger::enable_file(&file1).is_ok());
        assert_eq!(Logger::file(), true);
        info!("hello info");
        warn!("hello warn");
        assert_eq!(file1.exists(), true);
        assert!(Logger::disable_file().is_ok());
        assert_eq!(Logger::data().unwrap().len(), 0);
        let data: Vec<String> = sys::readstring(file1).unwrap().split("\n").map(|x| x.to_string()).collect();
        assert_eq!(data.len(), 3);
        assert!(data[0].starts_with("INFO ["));
        assert!(data[0].ends_with("hello info"));
        assert!(data[1].starts_with("WARN ["));
        assert!(data[1].ends_with("hello warn"));
        assert_eq!(data[2], "");

        assert!(sys::remove_all(&tmpdir).is_ok());
    }
}
