use rauc_health::config::HealthConfig;
use rauc_health::openrc::collect_failed_services;
use once_cell::sync::Lazy;

static CRONCFG: Lazy<HealthConfig> = Lazy::new (|| {
    HealthConfig {
        required_services:  vec!["cron".to_string()],
        ..Default::default()
    }
});

static SSHDCFG: Lazy<HealthConfig> = Lazy::new (|| {
    HealthConfig {
        required_services:  vec!["sshd".to_string()],
        ..Default::default()
    }
});

static TTYCFG: Lazy<HealthConfig> = Lazy::new (|| {
    HealthConfig {
        required_services:  vec!["getty.tty1".to_string()],
        ..Default::default()
    }
});

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
