use rauc_health::config::from_toml_str;
use rauc_health::openrc::{decide_health, HealthDecision};

#[test]
fn required_started_is_good() {
    let input = r#"
Runlevel: default
sshd [ started ]
foo  [ stopped ]
"#;

    let cfg = from_toml_str(r#"
required_services = ["sshd"]
"#).unwrap();

    let decision = decide_health(input, &cfg);
    assert!(matches!(decision, HealthDecision::Good));
}

#[test]
fn required_missing_is_bad() {
    let input = r#"
Runlevel: default
foo [ started ]
"#;

    let cfg = from_toml_str(r#"
required_services = ["sshd"]
"#).unwrap();

    let decision = decide_health(input, &cfg);
    assert!(matches!(decision, HealthDecision::Bad(_)));
}

#[test]
fn required_stopped_is_bad() {
    let input = r#"
Runlevel: default
sshd [ stopped ]
"#;

    let cfg = from_toml_str(r#"
required_services = ["sshd"]
"#).unwrap();

    let decision = decide_health(input, &cfg);
    assert!(matches!(decision, HealthDecision::Bad(_)));
}
