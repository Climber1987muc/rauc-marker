use rauc_health::openrc::{decide_health, HealthDecision};

#[test]
fn decision_is_good_when_no_failures() {
    let input = r#"
Runlevel: default
 sshd [ started ]
"#;

    let decision = decide_health(input);
    assert_eq!(decision, HealthDecision::Good);
}

#[test]
fn decision_is_bad_when_there_are_failures() {
    let input = r#"
Runlevel: default
 cron [ stopped ]
"#;

    let decision = decide_health(input);

    match decision {
        HealthDecision::Good => panic!("expected Bad, got Good"),
        HealthDecision::Bad(failed) => {
            assert!(failed.iter().any(|s| s.name == "cron" && s.status == "stopped"));
        }
    }
}
