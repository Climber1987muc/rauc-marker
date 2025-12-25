use anyhow::{Context, Result};
use clap::Parser;
use std::process::Command;

use cli::{Cli, Commands};


#[derive(Debug, Clone, PartialEq, Eq)]
struct FailedService {
    name: String,
    status: String,
}

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::MarkGood => rauc::mark_good()?,
        Commands::MarkBad => rauc::mark_bad()?,
        Commands::CheckOpenrc => openrc::heck_openrc_and_mark()?,
    }

    Ok(())
}

