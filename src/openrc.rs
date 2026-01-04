use crate::cli::CheckOpenrcArgs;
use crate::config::HealthConfig;
use std::collections::HashMap;
use std::process::Command;
use std::time::{Duration, Instant};

const STARTED: &str = "started";

/// Represents a required service that is not in the expected state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailedService {
    /// Name of the service.
    pub name: String,
    /// Reported service status (e.g. "stopped", "crashed", "missing").
    pub status: String,
}

/// Result of evaluating the `OpenRC` service health.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthDecision {
    /// All required services are running as expected.
    Good,
    /// One or more required services are missing or not started.
    Bad(Vec<FailedService>),
}

/// Wrapper that allows injecting a preloaded [`HealthConfig`].
///
/// This is mainly useful for tests or higher-level orchestration code.
///
/// # Errors
///
/// Returns any error produced by [`check_openrc_and_mark_with`].
pub fn check_openrc_and_mark_with_cfg(
    args: &CheckOpenrcArgs,
    cfg: &HealthConfig,
    deps: &OpenRcDeps<'_>,
) -> Result<(), String> {
    check_openrc_and_mark_with(args, cfg, deps)
}

/// Decides overall system health based on `rc-status` output.
///
/// # Returns
///
/// - [`HealthDecision::Good`] if all required services are started
/// - [`HealthDecision::Bad`] with a list of failed services otherwise
#[must_use]
pub fn decide_health(stdout: &str, cfg: &HealthConfig) -> HealthDecision {
    let failed = collect_failed_services(stdout, cfg);
    if failed.is_empty() {
        HealthDecision::Good
    } else {
        HealthDecision::Bad(failed)
    }
}

/// Collects all required services that are missing or not started.
///
/// # Behavior
///
/// - Services listed in `cfg.ignore_exact` or `cfg.ignore_prefixes` are skipped
/// - A service is considered healthy only if its status equals `"started"`
///
/// # Returns
///
/// A list of [`FailedService`] entries. The list is empty if all services are healthy.
#[must_use]
pub fn collect_failed_services(stdout: &str, cfg: &HealthConfig) -> Vec<FailedService> {
    let services = parse_services_map(stdout);
    let mut failed = Vec::new();

    for req in &cfg.required_services {
        if is_ignored_service(req, cfg) {
            continue;
        }

        match services.get(req) {
            Some(status) if status == STARTED => {}
            Some(status) => failed.push(FailedService {
                name: req.clone(),
                status: status.clone(),
            }),
            None => failed.push(FailedService {
                name: req.clone(),
                status: "missing".to_string(),
            }),
        }
    }

    failed
}

/// Checks whether a service should be ignored according to the configuration.
fn is_ignored_service(name: &str, cfg: &HealthConfig) -> bool {
    cfg.ignore_exact.iter().any(|s| s == name)
        || cfg.ignore_prefixes.iter().any(|p| name.starts_with(p))
}

/// Parses `rc-status` output into a service → status map.
///
/// # Behavior
///
/// - Ignores empty lines and runlevel headers
/// - Expects lines like: `cron [ started ]`
///
/// # Returns
///
/// A map where the key is the service name and the value is its status.
#[must_use]
pub fn parse_services_map(stdout: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for raw_line in stdout.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with("Runlevel:") || line.starts_with("Dynamic Runlevel:") {
            continue;
        }

        let mut it = line.split_whitespace();
        let Some(svc_name) = it.next() else { continue };
        let _ = it.next(); // usually "["
        let Some(status) = it.next() else { continue };

        map.insert(svc_name.to_string(), status.to_string());
    }

    map
}

//
// ============================
// Dependency Injection Layer
// ============================
//

/// Injectable dependencies for `OpenRC` health checks.
///
/// This allows deterministic testing by replacing system calls,
/// sleeping, and time measurement.
pub struct OpenRcDeps<'a> {
    /// Runs `rc-status` and returns its stdout.
    pub run_rc_status: &'a dyn Fn() -> Result<String, String>,
    /// Marks the system as healthy.
    pub mark_good: &'a dyn Fn() -> Result<(), String>,
    /// Marks the system as unhealthy.
    pub mark_bad: &'a dyn Fn() -> Result<(), String>,
    /// Sleeps for the given duration.
    pub sleep: &'a dyn Fn(Duration),
    /// Returns the current time.
    pub now: &'a dyn Fn() -> Instant,
}

/// Production entry point using real system dependencies.
///
/// # Errors
///
/// Returns `Err(String)` if:
/// - the configuration cannot be loaded
/// - `rc-status` fails or exits unsuccessfully
/// - marking the system good or bad fails
pub fn check_openrc_and_mark(args: &CheckOpenrcArgs) -> Result<(), String> {
    let cfg = match args.config.as_deref() {
        Some(path) => crate::config::from_file(path)?,
        None => HealthConfig::default(),
    };

    let deps = OpenRcDeps {
        run_rc_status: &|| {
            let output = Command::new("rc-status")
                .args(["--nocolor", "default"])
                .output()
                .map_err(|e| format!("failed to execute rc-status: {e}"))?;

            if !output.status.success() {
                return Err(format!("rc-status exited with {}", output.status));
            }

            String::from_utf8(output.stdout)
                .map_err(|_| "rc-status output was not valid UTF-8".to_string())
        },
        mark_good: &crate::rauc::mark_good,
        mark_bad: &crate::rauc::mark_bad,
        sleep: &std::thread::sleep,
        now: &Instant::now,
    };

    check_openrc_and_mark_with(args, &cfg, &deps)
}

/// Testable core logic for the `OpenRC` health check loop.
///
/// # Behavior
///
/// - Polls `rc-status` until all services are healthy or a timeout is reached
/// - Retries at `poll_interval_ms` intervals
/// - Marks the system good immediately on success
/// - Marks the system bad once the timeout expires
///
/// # Errors
///
/// Returns `Err(String)` if:
/// - querying `rc-status` fails
/// - the timeout expires before services become healthy
/// - marking the system good or bad fails
pub fn check_openrc_and_mark_with(
    args: &CheckOpenrcArgs,
    cfg: &HealthConfig,
    deps: &OpenRcDeps<'_>,
) -> Result<(), String> {
    let deadline = (deps.now)() + Duration::from_secs(args.timeout_secunds);

    loop {
        let stdout = (deps.run_rc_status)()?;

        match decide_health(&stdout, cfg) {
            HealthDecision::Good => {
                (deps.mark_good)()?;
                return Ok(());
            }
            HealthDecision::Bad(failed) => {
                if (deps.now)() < deadline {
                    (deps.sleep)(Duration::from_millis(args.poll_interval_ms));
                    continue;
                }

                (deps.mark_bad)()?;
                return Err(format!(
                    "OpenRC health check failed ({} failing services)",
                    failed.len()
                ));
            }
        }
    }
}
