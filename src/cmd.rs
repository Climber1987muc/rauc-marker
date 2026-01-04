// Datei: src/cmd.rs
use std::process::{Command, ExitStatus};

/// Abstraction for running external commands.
///
/// This trait makes command execution testable by allowing alternative
/// implementations (e.g., mocks) in unit tests.
pub trait CmdRunner {
    /// Runs `program` with `args` and returns its [`ExitStatus`].
    ///
    /// # Errors
    ///
    /// Returns `Err(String)` if the command could not be **spawned/executed**,
    /// for example when:
    /// - the executable does not exist / is not found in `PATH`
    /// - the process lacks permissions to execute it
    /// - an OS-level error prevents process creation
    ///
    /// The returned error string should include the provided `ctx` to describe
    /// what was being attempted.
    fn status(&self, program: &str, args: &[&str], ctx: &str) -> Result<ExitStatus, String>;
}

/// Real implementation of [`CmdRunner`] backed by [`std::process::Command`].
#[derive(Debug, Default, Clone, Copy)]
pub struct RealCmdRunner;

impl CmdRunner for RealCmdRunner {
    /// Executes the command on the host system and returns its [`ExitStatus`].
    ///
    /// # Errors
    ///
    /// Returns `Err(String)` if [`Command::status`] fails (i.e., the process
    /// could not be started). The error message includes `ctx`.
    fn status(&self, program: &str, args: &[&str], ctx: &str) -> Result<ExitStatus, String> {
        Command::new(program)
            .args(args)
            .status()
            .map_err(|e| format!("failed to execute `{ctx}`: {e}"))
    }
}

/// Runs a command and treats any non-success exit status as an error.
///
/// This is useful when a non-zero exit code should abort the current operation.
///
/// # Errors
///
/// Returns `Err(String)` if:
/// - the command could not be started (propagates the error from [`CmdRunner::status`])
/// - the command ran but exited unsuccessfully (non-zero exit status)
///
/// In both cases, the error message includes the provided `ctx`.
pub fn run_checked(
    runner: &dyn CmdRunner,
    program: &str,
    args: &[&str],
    ctx: &str,
) -> Result<(), String> {
    let status = runner.status(program, args, ctx)?;
    if !status.success() {
        return Err(format!("`{ctx}` exited with {status}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(unix)]
    fn real_runner_status_ok_on_true() {
        let r = RealCmdRunner;
        let st = r.status("true", &[], "true").unwrap();
        assert!(st.success());
    }

    #[test]
    #[cfg(unix)]
    fn real_runner_status_ok_on_false_but_not_success() {
        let r = RealCmdRunner;
        let st = r.status("false", &[], "false").unwrap();
        assert!(!st.success());
    }

    #[test]
    #[cfg(unix)]
    fn run_checked_ok_when_command_succeeds() {
        let r = RealCmdRunner;
        let res = run_checked(&r, "true", &[], "true");
        assert!(res.is_ok());
    }

    #[test]
    #[cfg(unix)]
    fn run_checked_err_when_command_fails() {
        let r = RealCmdRunner;
        let err = run_checked(&r, "false", &[], "false").unwrap_err();
        assert!(err.contains("`false` exited with"));
    }

    #[test]
    #[cfg(unix)]
    fn run_checked_err_when_command_missing() {
        let r = RealCmdRunner;
        let err =
            run_checked(&r, "__definitely_not_a_real_command__", &[], "missing cmd").unwrap_err();
        assert!(err.contains("failed to execute `missing cmd`"));
    }
}
