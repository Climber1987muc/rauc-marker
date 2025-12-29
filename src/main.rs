use clap::Parser;
use rauc_health::cli::Cli;
use rauc_health::cli::Commands;
use rauc_health::openrc;
use rauc_health::rauc;

fn main() -> Result<(), String> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::MarkGood => rauc::mark_good()?,
        Commands::MarkBad => rauc::mark_bad()?,
        Commands::CheckOpenrc(args) => openrc::check_openrc_and_mark(&args)?,
    }

    Ok(())
}
