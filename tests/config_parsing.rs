use rauc_health::config::from_toml_str;

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

    assert_eq!(cfg.required_services, vec!["sshd", "net.eth0"]);
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

    assert_eq!(cfg.ignore_exact, vec!["foo"]);
    assert_eq!(cfg.ignore_prefixes, vec!["bar."]);
}

#[test]
fn invalid_toml_is_error() {
    let err = from_toml_str("required_services = [").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("invalid config TOML"));
}
