use duct::cmd;

use clap::Parser;

mod commands;
use commands::Arguments;

mod logger;
use logger::enable_logging;

fn main() {
    let args = Arguments::parse();

    enable_logging(args.verbosity).unwrap();

    log::debug!("Debugging events enabled");
    log::trace!("Tracing events enabled");

    println!("{:#?}", args);
    println!("Hello, world!");
}
