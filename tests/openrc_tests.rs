// tests/openrc_tests.rs

use once_cell::sync::Lazy;

use rauc_health::cli::CheckOpenrcArgs;
use rauc_health::config::HealthConfig;
use rauc_health::openrc::{
    check_openrc_and_mark_with, check_openrc_and_mark_with_cfg, collect_failed_services,
    decide_health, parse_services_map, HealthDecision, OpenRcDeps,
};

use std::time::{Duration, Instant};

// ------------------------------------------------------------
// Static configs
// ------------------------------------------------------------

static CRONCFG: Lazy<HealthConfig> = Lazy::new(|| HealthConfig {
    required_services: vec!["cron".to_string()],
    ..Default::default()
});

static SSHDCFG: Lazy<HealthConfig> = Lazy::new(|| HealthConfig {
    required_services: vec!["sshd".to_string()],
    ..Default::default()
});

static TTYCFG: Lazy<HealthConfig> = Lazy::new(|| HealthConfig {
    required_services: vec!["getty.tty1".to_string()],
    ..Default::default()
});

static MULTICFG: Lazy<HealthConfig> = Lazy::new(|| HealthConfig {
    required_services: vec!["cron".into(), "sshd".into(), "dbus".into()],
    ..Default::default()
});

// ------------------------------------------------------------
// Helpers (args/cfg)
// ------------------------------------------------------------

fn args() -> CheckOpenrcArgs {
    CheckOpenrcArgs {
        config: None,
        timeout_secunds: 30,
        poll_interval_ms: 50,
    }
}

/// For timeout behavior, make it deterministic: 0 seconds timeout.
fn args_timeout_immediate() -> CheckOpenrcArgs {
    CheckOpenrcArgs {
        config: None,
        timeout_secunds: 0,
        poll_interval_ms: 1,
    }
}

fn cfg_required_cron() -> HealthConfig {
    HealthConfig {
        required_services: vec!["cron".into()],
        ..Default::default()
    }
}

// ------------------------------------------------------------
// Helper fns for deps (fn-items are 'static -> no lifetime issues)
// ------------------------------------------------------------

fn run_rc_ok() -> Result<String, String> {
    Ok("Runlevel: default\ncron [ started ]\n".to_string())
}

fn run_rc_bad() -> Result<String, String> {
    Ok("Runlevel: default\ncron [ stopped ]\n".to_string())
}

fn mark_ok() -> Result<(), String> {
    Ok(())
}

fn sleep_noop(_d: Duration) {}

fn now_real() -> Instant {
    Instant::now()
}

fn deps_ok<'a>() -> OpenRcDeps<'a> {
    OpenRcDeps {
        run_rc_status: &run_rc_ok,
        mark_good: &mark_ok,
        mark_bad: &mark_ok,
        sleep: &sleep_noop,
        now: &now_real,
    }
}

fn deps_bad<'a>() -> OpenRcDeps<'a> {
    OpenRcDeps {
        run_rc_status: &run_rc_bad,
        mark_good: &mark_ok,
        mark_bad: &mark_ok,
        sleep: &sleep_noop,
        now: &now_real,
    }
}

// ------------------------------------------------------------
// collect_failed_services tests
// ------------------------------------------------------------

#[test]
fn collects_non_started_required_services() {
    let input = r#"
Runlevel: default
cron [ stopped ]
sshd [ started ]
"#;

    let failed = collect_failed_services(input, &CRONCFG);

    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0].name, "cron");
    assert_eq!(failed[0].status, "stopped");
}

#[test]
fn ignores_non_required_services() {
    let input = r#"
Runlevel: default
cron [ stopped ]
sshd [ started ]
"#;

    let failed = collect_failed_services(input, &SSHDCFG);
    assert!(failed.is_empty());
}

#[test]
fn ignores_configured_ignored_services_even_if_required() {
    let input = r#"
Runlevel: default
getty.tty1 [ stopped ]
"#;

    let failed = collect_failed_services(input, &TTYCFG);
    assert!(failed.is_empty());
}

#[test]
fn ignores_unparseable_lines() {
    let input = r#"
Runlevel: default
this is garbage
cron [ stopped ]
"#;

    let failed = collect_failed_services(input, &CRONCFG);
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0].name, "cron");
}

#[test]
fn ignores_unparseable_lines_but_keeps_parsing() {
    let input = r#"
Runlevel: default
### nonsense ###
cron [ stopped ]
more garbage here
"#;

    let failed = collect_failed_services(input, &CRONCFG);
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0].name, "cron");
}

#[test]
fn marks_missing_required_services_as_missing() {
    let input = r#"
Runlevel: default
cron [ started ]
"#;

    let cfg = HealthConfig {
        required_services: vec!["sshd".into()],
        ..Default::default()
    };

    let failed = collect_failed_services(input, &cfg);

    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0].name, "sshd");
    assert_eq!(failed[0].status, "missing");
}

#[test]
fn started_required_service_is_not_failed() {
    let input = r#"
Runlevel: default
cron [ started ]
"#;

    let cfg = HealthConfig {
        required_services: vec!["cron".into()],
        ..Default::default()
    };

    let failed = collect_failed_services(input, &cfg);
    assert!(failed.is_empty());
}

#[test]
fn collects_multiple_failures_and_missing() {
    let input = r#"
Runlevel: default
cron [ stopped ]
sshd [ started ]
"#;

    let failed = collect_failed_services(input, &MULTICFG);

    assert_eq!(failed.len(), 2);
    assert!(failed
        .iter()
        .any(|f| f.name == "cron" && f.status == "stopped"));
    assert!(failed
        .iter()
        .any(|f| f.name == "dbus" && f.status == "missing"));
}

#[test]
fn empty_required_services_yields_no_failures() {
    let input = r#"
Runlevel: default
cron [ stopped ]
"#;

    let cfg = HealthConfig {
        required_services: vec![],
        ..Default::default()
    };

    let failed = collect_failed_services(input, &cfg);
    assert!(failed.is_empty());
}

#[test]
fn status_is_case_sensitive_by_current_logic() {
    let input = r#"
Runlevel: default
cron [ Started ]
"#;

    let cfg = HealthConfig {
        required_services: vec!["cron".into()],
        ..Default::default()
    };

    let failed = collect_failed_services(input, &cfg);

    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0].status, "Started");
}

// ------------------------------------------------------------
// parse_services_map tests
// ------------------------------------------------------------

#[test]
fn parse_services_map_skips_empty_and_runlevel_lines() {
    let input = r#"

Runlevel: default
Dynamic Runlevel: hotplugged
cron [ started ]
"#;

    let m = parse_services_map(input);
    assert_eq!(m.get("cron").map(String::as_str), Some("started"));
    assert_eq!(m.len(), 1);
}

#[test]
fn parse_services_map_skips_lines_with_too_few_tokens() {
    let input = r#"
Runlevel: default
cron
sshd [
getty.tty1 [
cron [ started ]
"#;

    let m = parse_services_map(input);
    assert_eq!(m.len(), 1);
    assert_eq!(m.get("cron").map(String::as_str), Some("started"));
}

#[test]
fn parse_services_map_handles_tabs_and_extra_spaces() {
    let input = "cron\t[\tstarted\t]\n";
    let m = parse_services_map(input);
    assert_eq!(m.get("cron").map(String::as_str), Some("started"));
}

#[test]
fn parse_services_map_overwrites_on_duplicate_service_names() {
    let input = r#"
cron [ stopped ]
cron [ started ]
"#;

    let m = parse_services_map(input);
    assert_eq!(m.get("cron").map(String::as_str), Some("started"));
}

// ------------------------------------------------------------
// decide_health tests
// ------------------------------------------------------------

#[test]
fn decide_health_good_when_no_failures() {
    let input = "cron [ started ]\n";
    let cfg = cfg_required_cron();

    let d = decide_health(input, &cfg);
    assert!(matches!(d, HealthDecision::Good));
}

#[test]
fn decide_health_bad_contains_failed_services() {
    let input = "cron [ stopped ]\n";
    let cfg = cfg_required_cron();

    let d = decide_health(input, &cfg);
    match d {
        HealthDecision::Bad(f) => {
            assert_eq!(f.len(), 1);
            assert_eq!(f[0].name, "cron");
        }
        _ => panic!("expected Bad"),
    }
}

// ------------------------------------------------------------
// check_openrc_and_mark_* tests (deps injection)
// ------------------------------------------------------------

#[test]
fn check_openrc_marks_good() {
    let res = check_openrc_and_mark_with(&args(), &cfg_required_cron(), &deps_ok());
    assert!(res.is_ok());
}

#[test]
fn check_openrc_timeout_errors() {
    // deterministic: always BAD + timeout=0 => immediate mark_bad + Err
    let err =
        check_openrc_and_mark_with(&args_timeout_immediate(), &cfg_required_cron(), &deps_bad())
            .unwrap_err();

    assert!(err.contains("OpenRC health check failed"));
}

#[test]
fn check_openrc_cfg_variant_works() {
    let res = check_openrc_and_mark_with_cfg(&args(), &cfg_required_cron(), &deps_ok());
    assert!(res.is_ok());
}
