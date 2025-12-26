use anyhow::Result;
use rauc_health::openrc;
use rauc_health::rauc;
use rauc_health::cli::Commands;
use rauc_health::cli::Cli;
use clap::Parser;

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::MarkGood => rauc::mark_good()?,
        Commands::MarkBad => rauc::mark_bad()?,
        Commands::CheckOpenrc(_args) => openrc::check_openrc_and_mark()?,
    }

    Ok(())
}

