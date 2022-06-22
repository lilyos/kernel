use log::{error, Level, LevelFilter, Log, Metadata, Record};

/// `Log` implementation for serial
pub struct SerialLogger;

impl Log for SerialLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Self::LEVEL
    }

    fn log(&self, record: &Record) {
        println!(
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
        );
    }

    fn flush(&self) {}
}

/// The static logger
pub static LOGGER: SerialLogger = SerialLogger;

impl SerialLogger {
    #[cfg(debug_assertions)]
    const LEVEL: Level = Level::Trace;
    #[cfg(debug_assertions)]
    const LEVEL_FILTER: LevelFilter = LevelFilter::Trace;

    #[cfg(not(debug_assertions))]
    const LEVEL: Level = Level::Info;
    #[cfg(not(debug_assertions))]
    const LEVEL_FILTER: LevelFilter = LevelFilter::Info;

    /// Initialize the logger
    pub fn init(&'static self) {
        match log::set_logger(self) {
            Ok(_) => log::set_max_level(Self::LEVEL_FILTER),
            Err(e) => panic!("FAILED TO INITIALIZE LOGGER, {e}"),
        }
    }
}
