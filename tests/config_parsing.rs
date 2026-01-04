// Datei: tests/config_tests.rs
//
// Läuft als Integration-Test (cargo test).
// Enthält DEINE bestehenden Tests + zusätzliche Tests für from_file und optional_services.

use rauc_health::config::{from_file, from_toml_str};
use std::fs;
use std::path::Path;

#[test]
fn empty_toml_uses_defaults() {
    let cfg = from_toml_str("").unwrap();

    // required/optional defaulten auf leer
    assert!(cfg.required_services.is_empty());
    assert!(cfg.optional_services.is_empty());

    // ignore defaults sind gesetzt
    assert!(cfg.ignore_exact.contains(&"local".to_string()));
    assert!(cfg.ignore_exact.contains(&"time-first-boot".to_string()));
    assert!(cfg.ignore_prefixes.contains(&"getty.".to_string()));
}

#[test]
fn partial_toml_overrides_required_only() {
    let cfg = from_toml_str(
        r#"
required_services = ["sshd", "net.eth0"]
"#,
    )
    .unwrap();

    assert_eq!(
        cfg.required_services,
        vec!["sshd".to_string(), "net.eth0".to_string()]
    );
    // ignore defaults bleiben erhalten
    assert!(cfg.ignore_exact.contains(&"local".to_string()));
}

#[test]
fn ignore_can_be_overridden() {
    let cfg = from_toml_str(
        r#"
ignore_exact = ["foo"]
ignore_prefixes = ["bar."]
"#,
    )
    .unwrap();

    assert_eq!(cfg.ignore_exact, vec!["foo".to_string()]);
    assert_eq!(cfg.ignore_prefixes, vec!["bar.".to_string()]);
}

#[test]
fn invalid_toml_is_error() {
    let err = from_toml_str("required_services = [").unwrap_err();
    assert_eq!(err, "invalid config TOML");
}

#[test]
fn optional_services_can_be_set_independently() {
    let cfg = from_toml_str(r#"optional_services = ["dbus"]"#).unwrap();

    assert!(cfg.required_services.is_empty());
    assert_eq!(cfg.optional_services, vec!["dbus".to_string()]);
    // ignore defaults bleiben
    assert!(cfg.ignore_prefixes.contains(&"agetty.".to_string()));
}

#[test]
fn from_file_reads_and_parses() {
    let mut p = std::env::temp_dir();
    p.push(format!("rauc_health_cfg_ok_{}.toml", std::process::id()));

    fs::write(&p, r#"required_services = ["cron"]"#).unwrap();

    let cfg = from_file(&p).unwrap();
    assert_eq!(cfg.required_services, vec!["cron".to_string()]);

    let _ = fs::remove_file(&p);
}

#[test]
fn from_file_missing_is_error() {
    let p = Path::new("/definitely/does/not/exist/rauc_health.toml");
    let err = from_file(p).unwrap_err();
    assert!(err.contains("failed to read config file"));
    assert!(err.contains("rauc_health.toml"));
}

#[test]
fn from_file_invalid_toml_is_error() {
    let mut p = std::env::temp_dir();
    p.push(format!("rauc_health_cfg_bad_{}.toml", std::process::id()));

    fs::write(&p, "required_services = [").unwrap();

    let err = from_file(&p).unwrap_err();
    assert!(err.contains("failed to parse config file"));
    assert!(err.contains(p.to_string_lossy().as_ref()));

    let _ = fs::remove_file(&p);
}
