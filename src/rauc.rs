use std::process::Command;

fn run_checked(programm: &str, args: &[&str], ctx: &str) -> Result<(), String> {
    let status = Command::new(programm)
        .args(args)
        .status()
        .map_err(|e| format!("failed to execute `{ctx}`: {e}"))?;

    if !status.success() {
        return Err(format!("`{ctx}` exited with {status}"));
    }

    Ok(())
}

/// Markiert den aktuellen Slot als gut (good).
///
/// # Errors
///
/// Diese Funktion gibt einen Fehler zurück, wenn das `rauc` Kommando
/// nicht ausgeführt werden kann oder mit einem Fehlerstatus beendet wird.
pub fn mark_good() -> Result<(), String> {
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
pub fn mark_bad() -> Result<(), String> {
    log::warn!("Marking current RAUC slot as BAD…");
    run_checked("rauc", &["status", "mark-bad"], "rauc status mark-bad")?;
    log::warn!("Successfully marked slot as BAD.");
    Ok(())
}
