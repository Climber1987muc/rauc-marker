use crate::cmd::{run_checked, CmdRunner, RealCmdRunner};

/// Markiert den aktuellen Slot als gut (good).
pub fn mark_good() -> Result<(), String> {
    mark_good_with(&RealCmdRunner)
}

/// Markiert den aktuellen Slot als fehlerhaft (bad).
pub fn mark_bad() -> Result<(), String> {
    mark_bad_with(&RealCmdRunner)
}

/// Testbare Variante von `mark_good`
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

/// Testbare Variante von `mark_bad`
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::ExitStatus;
    use std::sync::Mutex;

    // Hilfsfunktion für ExitStatus (Unix)
    #[cfg(unix)]
    fn exit_status(code: i32) -> ExitStatus {
        use std::os::unix::process::ExitStatusExt;
        ExitStatusExt::from_raw(code << 8)
    }

    // -----------------------------
    // Fake CmdRunner für Tests
    // -----------------------------
    struct FakeRunner {
        calls: Mutex<Vec<(String, Vec<String>, String)>>,
        next: Mutex<Result<ExitStatus, String>>,
    }

    impl Default for FakeRunner {
        fn default() -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
                next: Mutex::new(Ok(exit_status(0))), // default: Erfolg
            }
        }
    }

    impl FakeRunner {
        fn will_return(&self, res: Result<ExitStatus, String>) {
            *self.next.lock().unwrap() = res;
        }

        fn take_calls(&self) -> Vec<(String, Vec<String>, String)> {
            std::mem::take(&mut *self.calls.lock().unwrap())
        }
    }

    impl CmdRunner for FakeRunner {
        fn status(&self, program: &str, args: &[&str], ctx: &str) -> Result<ExitStatus, String> {
            self.calls.lock().unwrap().push((
                program.to_string(),
                args.iter().map(|s| s.to_string()).collect(),
                ctx.to_string(),
            ));
            self.next.lock().unwrap().clone()
        }
    }

    // -----------------------------
    // Tests
    // -----------------------------

    #[test]
    #[cfg(unix)]
    fn mark_good_calls_rauc_with_expected_args() {
        let runner = FakeRunner::default();

        mark_good_with(&runner).unwrap();

        let calls = runner.take_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "rauc");
        assert_eq!(
            calls[0].1,
            vec!["status".to_string(), "mark-good".to_string()]
        );
        assert_eq!(calls[0].2, "rauc status mark-good");
    }

    #[test]
    #[cfg(unix)]
    fn mark_bad_calls_rauc_with_expected_args() {
        let runner = FakeRunner::default();

        mark_bad_with(&runner).unwrap();

        let calls = runner.take_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(
            calls[0].1,
            vec!["status".to_string(), "mark-bad".to_string()]
        );
        assert_eq!(calls[0].2, "rauc status mark-bad");
    }

    #[test]
    #[cfg(unix)]
    fn non_zero_exit_is_error() {
        let runner = FakeRunner::default();
        runner.will_return(Ok(exit_status(1)));

        let err = mark_good_with(&runner).unwrap_err();
        assert!(err.contains("exited with"));
        assert!(err.contains("rauc status mark-good"));
    }

    #[test]
    #[cfg(unix)]
    fn execution_error_is_propagated() {
        let runner = FakeRunner::default();
        runner.will_return(Err("boom".to_string()));

        let err = mark_bad_with(&runner).unwrap_err();
        assert_eq!(err, "boom");
    }
}
