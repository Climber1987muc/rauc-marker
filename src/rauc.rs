use anyhow::{Context, Result};
use std::process::Command;

fn run_checked(programm: &str, args: &[&str], ctx: &str) -> Result<()> {
    let status = Command::new(programm)
        .args(args)
        .status()
        .with_context(|| format!("failed to execute `{}`", ctx))?;

    if !status.success() {
        anyhow::bail!("`{}` exited with {}", ctx, status);
    }

    Ok(())
}

pub fn mark_good() -> Result<()> {
    log::info!("Marking current RAUC slot as GOOD…");
    run_checked("rauc", &["status", "mark-good"], "rauc status mark-good")?;
    log::info!("Successfully marked slot as GOOD.");
    Ok(())
}

pub fn mark_bad() -> Result<()> {
    log::warn!("Marking current RAUC slot as BAD…");
    run_checked("rauc", &["status", "mark-bad"], "rauc status mark-bad")?;
    log::warn!("Successfully marked slot as BAD.");
    Ok(())
}
