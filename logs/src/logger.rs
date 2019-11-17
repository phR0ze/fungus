use chrono;
use colored::*;
use core::*;
use log;
use std::path::PathBuf;

// Initialize logger
// -------------------------------------------------------------------------------------------------

// Initialize the logger with default values, i.e. log::Level::Info.
pub fn init() -> Result<()> {
    init_with(vec![])
}

// Initialize the logger with the given options.
pub fn init_with(opts: Vec<Opt>) -> Result<()> {
    let level = opts.level();
    let logger = Logger { level };
    let _ = log::set_boxed_logger(Box::new(logger));
    log::set_max_level(level.to_level_filter());
    Ok(())
}

// Logger options
// -------------------------------------------------------------------------------------------------
#[derive(Clone)]
pub enum Opt {
    Level(log::Level),
    Filepath(PathBuf),
}
pub trait OptsExt {
    fn level(&self) -> log::Level;
}
impl OptsExt for Vec<Opt> {
    fn level(&self) -> log::Level {
        for opt in self {
            if let Opt::Level(level) = opt.clone() {
                return level;
            }
        }
        log::Level::Info
    }
}

// Logger
// -------------------------------------------------------------------------------------------------
pub struct Logger {
    level: log::Level,
}
impl log::Log for Logger {
    // Filter out incorrect levels
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.level
    }

    // Log with correct level color and timestamp
    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let level = {
                match record.level() {
                    log::Level::Error => record.level().to_string().red(),
                    log::Level::Warn => record.level().to_string().yellow(),
                    log::Level::Info => record.level().to_string().cyan(),
                    log::Level::Debug => record.level().to_string().normal(),
                    log::Level::Trace => record.level().to_string().normal().dimmed(),
                }
            };
            println!("{}[{}] {}", level, chrono::Local::now().format("%Y-%m-%d %H:%M:%S.%3f"), record.args());
        }
    }

    fn flush(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_init() {
    //     init().unwrap();
    //     log::error!("hello error");
    //     log::warn!("hello warn");
    //     log::info!("hello info");
    //     log::debug!("hello debug");
    //     log::trace!("hello trace");
    // }

    #[test]
    fn test_init_with() {
        init_with(vec![Opt::Level(log::Level::Trace)]).unwrap();
        log::error!("hello error");
        log::warn!("hello warn");
        log::info!("hello info");
        log::debug!("hello debug");
        log::trace!("hello trace");
    }
}
