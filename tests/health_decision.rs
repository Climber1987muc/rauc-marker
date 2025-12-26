use rauc_health::openrc::{decide_health, HealthDecision};
use rauc_health::config::HealthConfig;

#[test]
fn decision_is_good_when_no_failures() {
    let input = r#"
Runlevel: default
sshd [ started ]
"#;

    let mut cfg = HealthConfig::default();
    cfg.required_services = vec!["sshd".to_string()];

    let decision = decide_health(input, &cfg);
    assert_eq!(decision, HealthDecision::Good);
}

#[test]
fn decision_is_bad_when_there_are_failures() {
    let input = r#"
Runlevel: default
cron [ stopped ]
"#;

    let mut cfg = HealthConfig::default();
    cfg.required_services = vec!["cron".to_string()];

    let decision = decide_health(input, &cfg);

    match decision {
        HealthDecision::Good => panic!("expected Bad, got Good"),
        HealthDecision::Bad(failed) => {
            assert!(failed.iter().any(|s| s.name == "cron" && s.status == "stopped"));
        }
    }
}