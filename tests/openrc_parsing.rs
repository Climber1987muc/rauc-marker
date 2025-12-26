use rauc_health::openrc::collect_failed_services;
use rauc_health::config::HealthConfig;

#[test]
fn collects_non_started_services_and_ignores_configured_ones() {
    let input = r#"
Runlevel: default
 sshd        [ started ]
 cron        [ stopped ]
 getty.tty1  [ stopped ]
 local       [ stopped ]
Dynamic Runlevel: hotplugged
"#;

    
    let cfg = HealthConfig::default();
    let failed = collect_failed_services(input, &cfg);
    assert!(failed.iter().any(|s| s.name == "cron" && s.status == "stopped"));
    assert!(!failed.iter().any(|s| s.name == "getty.tty1"));
    assert!(!failed.iter().any(|s| s.name == "local"));
}


#[test]
fn no_failures_when_all_started() {
    let input = r#"
Runlevel: default
 sshd   [ started ]
 cron   [ started ]
"#;

   let cfg = HealthConfig::default();
    let failed = collect_failed_services(input, &cfg);
    assert!(failed.is_empty());
}


#[test]
fn ignores_headers_and_empty_lines() {
    let input = r#"

Runlevel: default

Dynamic Runlevel: hotplugged

 sshd   [ started ]
"#;

    let cfg = HealthConfig::default();
    let failed = collect_failed_services(input, &cfg);
    assert!(failed.is_empty());
}


#[test]
fn ignores_unparseable_lines() {
    let input = r#"
Runlevel: default
this line is weird
sshd [ started ]
cron [ stopped ]
"#;

    let cfg = HealthConfig::default();
    let failed = collect_failed_services(input, &cfg);
    assert!(failed.iter().any(|s| s.name == "cron" && s.status == "stopped"));
}
#[test]
fn ignores_exact_services() {
    let input = r#"
Runlevel: default
 local          [ stopped ]
 time-first-boot [ stopped ]
"#;

    let cfg = HealthConfig::default();
    let failed = collect_failed_services(input, &cfg);
    assert!(failed.is_empty());
}

#[test]
fn ignores_unparseable_lines_but_keeps_parsing() {
    let input = r#"
Runlevel: default
this line is weird and has no brackets
sshd [ started ]
cron [ stopped ]
"#;

    let cfg = HealthConfig::default();
    let failed = collect_failed_services(input, &cfg);
    assert!(failed.iter().any(|s| s.name == "cron" && s.status == "stopped"));
}
