use rauc_health::openrc::collect_failed_services;

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

    let failed = collect_failed_services(input);

    assert!(failed.iter().any(|s| s.name == "cron" && s.status == "stopped"));
    assert!(!failed.iter().any(|s| s.name == "getty.tty1"));
    assert!(!failed.iter().any(|s| s.name == "local"));
}
