// tests/rauc_test.rs

use std::process::ExitStatus;
use std::sync::Mutex;

use rauc_health::cmd::CmdRunner;
use rauc_health::rauc::{mark_bad_with, mark_good_with};

/// Helper function to construct an [`ExitStatus`] on Unix systems.
#[cfg(unix)]
fn exit_status(code: i32) -> ExitStatus {
    use std::os::unix::process::ExitStatusExt;
    ExitStatusExt::from_raw(code << 8)
}

// ------------------------------------------------------------
// Fake CmdRunner
// ------------------------------------------------------------

struct FakeRunner {
    calls: Mutex<Vec<(String, Vec<String>, String)>>,
    next: Mutex<Result<ExitStatus, String>>,
}

impl Default for FakeRunner {
    fn default() -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            next: Mutex::new(Ok(exit_status(0))),
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

// ------------------------------------------------------------
// Tests
// ------------------------------------------------------------

#[test]
#[cfg(unix)]
fn mark_good_calls_rauc_with_expected_args() {
    let runner = FakeRunner::default();

    mark_good_with(&runner).unwrap();

    let calls = runner.take_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "rauc");
    assert_eq!(calls[0].1, vec!["status", "mark-good"]);
    assert_eq!(calls[0].2, "rauc status mark-good");
}

#[test]
#[cfg(unix)]
fn mark_bad_calls_rauc_with_expected_args() {
    let runner = FakeRunner::default();

    mark_bad_with(&runner).unwrap();

    let calls = runner.take_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].1, vec!["status", "mark-bad"]);
    assert_eq!(calls[0].2, "rauc status mark-bad");
}

#[test]
#[cfg(unix)]
fn non_zero_exit_is_error() {
    let runner = FakeRunner::default();
    runner.will_return(Ok(exit_status(1)));

    let err = mark_good_with(&runner).unwrap_err();
    assert!(err.contains("exited with"));
}

#[test]
#[cfg(unix)]
fn execution_error_is_propagated() {
    let runner = FakeRunner::default();
    runner.will_return(Err("boom".to_string()));

    let err = mark_bad_with(&runner).unwrap_err();
    assert_eq!(err, "boom");
}
