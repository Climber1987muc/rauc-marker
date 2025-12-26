use rauc_health::config::from_file;
use std::io::Write;

#[test]
fn loads_config_from_file() {
    let mut cfgfile = tempfile::NamedTempFile::new().unwrap();
    write!(cfgfile, r#"required_services = ["sshd"]"#).unwrap();

    let cfg = from_file(cfgfile.path()).unwrap();
    assert_eq!(cfg.required_services, vec!["sshd"]);
}
