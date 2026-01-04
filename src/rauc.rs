use crate::cmd::{run_checked, CmdRunner, RealCmdRunner};

/// Marks the currently active RAUC slot as **good**.
///
/// This is the production entry point using the real command runner.
///
/// # Errors
///
/// Returns `Err(String)` if invoking the `rauc` command fails or if the command
/// exits with a non-zero status.
pub fn mark_good() -> Result<(), String> {
    mark_good_with(&RealCmdRunner)
}

/// Marks the currently active RAUC slot as **bad**.
///
/// This is the production entry point using the real command runner.
///
/// # Errors
///
/// Returns `Err(String)` if invoking the `rauc` command fails or if the command
/// exits with a non-zero status.
pub fn mark_bad() -> Result<(), String> {
    mark_bad_with(&RealCmdRunner)
}

/// Testable variant of [`mark_good`] using an injected [`CmdRunner`].
///
/// # Behavior
///
/// Executes:
///
/// ```text
/// rauc status mark-good
/// ```
///
/// # Errors
///
/// Returns `Err(String)` if:
/// - the command cannot be executed
/// - the command exits with a non-zero exit status
pub fn mark_good_with(runner: &dyn CmdRunner) -> Result<(), String> {
    log::info!("Marking current RAUC slot as GOOD…");
    run_checked(
        runner,
        "rauc",
        &["status", "mark-good"],
        "rauc status mark-good",
    )?;
    log::info!("Successfully marked slot as GOOD.");
    Ok(())
}

/// Testable variant of [`mark_bad`] using an injected [`CmdRunner`].
///
/// # Behavior
///
/// Executes:
///
/// ```text
/// rauc status mark-bad
/// ```
///
/// # Errors
///
/// Returns `Err(String)` if:
/// - the command cannot be executed
/// - the command exits with a non-zero exit status
pub fn mark_bad_with(runner: &dyn CmdRunner) -> Result<(), String> {
    log::warn!("Marking current RAUC slot as BAD…");
    run_checked(
        runner,
        "rauc",
        &["status", "mark-bad"],
        "rauc status mark-bad",
    )?;
    log::warn!("Successfully marked slot as BAD.");
    Ok(())
}
