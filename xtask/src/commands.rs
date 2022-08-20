use std::str::FromStr;

use anyhow::bail;
use clap::{Parser, Subcommand};

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

#[derive(Debug)]
pub enum Target {
    X86_64,
}

impl FromStr for Target {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "x86_64" => Ok(Self::X86_64),
            _ => bail!("Invalid target specified"),
        }
    }
}

#[derive(Subcommand, Debug)]
/// Manage tools required for the kernel
pub enum ToolAction {
    List {},
    Install { to_add: Vec<String> },
    Uninstall { to_remove: Vec<String> },
}
