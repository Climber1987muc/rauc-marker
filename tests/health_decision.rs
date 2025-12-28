use once_cell::sync::Lazy;
use rauc_health::config::HealthConfig;
use rauc_health::openrc::{decide_health, HealthDecision};

static CRONCFG: Lazy<HealthConfig> = Lazy::new(|| HealthConfig {
    required_services: vec!["cron".to_string()],
    ..Default::default()
});

static SSHDCFG: Lazy<HealthConfig> = Lazy::new(|| HealthConfig {
    required_services: vec!["sshd".to_string()],
    ..Default::default()
});

#[test]
fn decision_is_good_when_no_failures() {
    let input = r#"
Runlevel: default
sshd [ started ]
"#;

    let decision = decide_health(input, &SSHDCFG);
    assert_eq!(decision, HealthDecision::Good);
}

#[test]
fn decision_is_bad_when_there_are_failures() {
    let input = r#"
Runlevel: default
cron [ stopped ]
"#;

    let decision = decide_health(input, &CRONCFG);

    match decision {
        HealthDecision::Good => panic!("expected Bad, got Good"),
        HealthDecision::Bad(failed) => {
            assert!(failed
                .iter()
                .any(|s| s.name == "cron" && s.status == "stopped"));
        }
    }
}
