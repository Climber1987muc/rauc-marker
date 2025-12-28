use anyhow::{Context, Result};
use std::process::Command;

fn run_checked(programm: &str, args: &[&str], ctx: &str) -> Result<()> {
    let status = Command::new(programm)
        .args(args)
        .status()
        .with_context(|| format!("failed to execute `{ctx}`"))?;

    if !status.success() {
        anyhow::bail!("`{ctx}` exited with {status}");
    }

    Ok(())
}

/// Markiert den aktuellen Slot als gut (good).
///
/// # Errors
///
/// Diese Funktion gibt einen Fehler zurück, wenn das `rauc` Kommando
/// nicht ausgeführt werden kann oder mit einem Fehlerstatus beendet wird.
pub fn mark_good() -> Result<()> {
    log::info!("Marking current RAUC slot as GOOD…");
    run_checked("rauc", &["status", "mark-good"], "rauc status mark-good")?;
    log::info!("Successfully marked slot as GOOD.");
    Ok(())
}

/// Markiert den aktuellen Slot als fehlerhaft (bad).
///
/// # Errors
///
/// Diese Funktion gibt einen Fehler zurück, wenn das `rauc` Kommando
/// nicht ausgeführt werden kann oder mit einem Fehlerstatus beendet wird.
pub fn mark_bad() -> Result<()> {
    log::warn!("Marking current RAUC slot as BAD…");
    run_checked("rauc", &["status", "mark-bad"], "rauc status mark-bad")?;
    log::warn!("Successfully marked slot as BAD.");
    Ok(())
}
