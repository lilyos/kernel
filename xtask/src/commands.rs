use clap::{Parser, Subcommand};

use crate::builder::Target;

#[derive(Parser, Debug)]
#[clap(author = "Jess B <jessmakesmeshes@gmail.com>", version, about = "XTask runner for the Lotus kernel", long_about = None)]
pub struct Arguments {
    #[clap(short, long)]
    /// Build or test in release mode
    pub release: bool,
    /// Verbosity
    #[clap(short, long, parse(from_occurrences))]
    pub verbosity: usize,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Manage tools
    Tools {
        #[clap(subcommand)]
        action: ToolAction,
    },
    Build {
        #[clap(short, long)]
        target: Target,
    },
}

#[derive(Subcommand, Debug)]
/// Manage tools required for the kernel
pub enum ToolAction {
    List {},
    Install { to_add: Vec<String> },
    Uninstall { to_remove: Vec<String> },
}
