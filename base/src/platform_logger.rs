use core::fmt::Write;

use log::{error, Level, LevelFilter, Log, Metadata, Record, SetLoggerError};

use crate::{arch::PLATFORM_MANAGER, traits::Platform};

/// `Log` implementation for serial
pub struct PlatformLogger;

impl Log for PlatformLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Self::LEVEL
    }

    fn log(&self, record: &Record) {
        writeln!(
            PLATFORM_MANAGER.get_text_output(),
            "[{}:{}] {}: {}",
            match record.file() {
                Some(v) => v,
                None => {
                    error!("FILE NAME NOT PRESENT");
                    "???"
                }
            },
            match record.line() {
                Some(v) => v,
                None => {
                    error!("FILE LINE NOT PRESENT");
                    0
                }
            },
            record.level(),
            record.args()
        ).unwrap();
    }

    fn flush(&self) {}
}

/// The static logger
pub static LOGGER: PlatformLogger = PlatformLogger;

impl PlatformLogger {
    #[cfg(debug_assertions)]
    const LEVEL: Level = Level::Trace;
    #[cfg(debug_assertions)]
    const LEVEL_FILTER: LevelFilter = LevelFilter::Trace;

    #[cfg(not(debug_assertions))]
    const LEVEL: Level = Level::Info;
    #[cfg(not(debug_assertions))]
    const LEVEL_FILTER: LevelFilter = LevelFilter::Info;

    /// Initialize the logger
    pub fn init(&'static self) -> Result<(), SetLoggerError> {
        log::set_logger(self)?;
        log::set_max_level(Self::LEVEL_FILTER);
        Ok(())
    }
}
