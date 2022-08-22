use anyhow::Result;

use clap::Parser;

mod builder;
mod commands;
mod logger;
mod tools;

use crate::{
    commands::Arguments,
    logger::enable_logging,
    tools::{get_tools, install_tools, print_tools, uninstall_tools},
};

fn main() -> Result<()> {
    let args = Arguments::parse();

    enable_logging(args.verbosity).unwrap();

    log::debug!("Debugging events enabled");
    log::trace!("Tracing events enabled");

    let tools = get_tools();

    match args.command {
        commands::Command::Tools { ref action } => match action {
            commands::ToolAction::List {} => print_tools(&tools),
            commands::ToolAction::Install { to_add } => install_tools(to_add.as_slice())?,
            commands::ToolAction::Uninstall { to_remove } => uninstall_tools(to_remove.as_slice())?,
        },
        commands::Command::Build { ref target } => todo!(),
    }

    println!("{:#?}", args);
    println!("Hello, world!");
    Ok(())
}
