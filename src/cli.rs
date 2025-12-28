use clap::Args;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "rauc-health")]
#[command(about = "Simple RAUC health helper for OpenRC", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Mark the currently booted RAUC slot as good
    MarkGood,
    /// Mark the currently booted RAUC slot as bad
    MarkBad,
    /// Check `OpenRC` runlevel 'default' and mark RAUC slot good/bad
    CheckOpenrc(CheckOpenrcArgs),
}
#[derive(Args, Debug, Clone)]
pub struct CheckOpenrcArgs {
    #[arg(long)]
    pub config: Option<PathBuf>,

    #[arg(long, default_value_t = 30)]
    pub timeout_secunds: u64,

    #[arg(long, default_value_t = 500)]
    pub poll_interval_ms: u64,
}
