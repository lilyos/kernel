use std::str::FromStr;

use anyhow::bail;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(author = "Jess B <jessmakesmeshes@gmail.com>", version, about = "XTask runner for the Lotus kernel", long_about = None)]
pub struct Arguments {
    #[clap(short, long)]
    /// Build or test in release mode
    pub release: bool,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
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

fn main() {
    let args = Arguments::parse();

    println!("{:#?}", args);

    println!("Hello, world!");
}
