// Datei: src/cmd.rs
use std::process::{Command, ExitStatus};

pub trait CmdRunner {
    fn status(&self, program: &str, args: &[&str], ctx: &str) -> Result<ExitStatus, String>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RealCmdRunner;

impl CmdRunner for RealCmdRunner {
    fn status(&self, program: &str, args: &[&str], ctx: &str) -> Result<ExitStatus, String> {
        Command::new(program)
            .args(args)
            .status()
            .map_err(|e| format!("failed to execute `{ctx}`: {e}"))
    }
}

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
