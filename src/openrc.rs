use anyhow::{Context, Result};
use std::process::Command;
use std::collections::HashMap;
use crate::config::HealthConfig;

const STARTED: &str = "started";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailedService {
    pub name: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthDecision {
    Good,
    Bad(Vec<FailedService>),
}


pub fn decide_health(stdout: &str, cfg: &HealthConfig) -> HealthDecision {
    let failed = collect_failed_services(stdout, cfg);
    if failed.is_empty() {
        HealthDecision::Good
    } else {
        HealthDecision::Bad(failed)
    }
}

pub fn collect_failed_services(stdout: &str, cfg: &HealthConfig) -> Vec<FailedService> {
    let services = parse_services_map(stdout);
    let mut failed = Vec::new();

    // Wenn required leer ist, würde sonst immer "Good" rauskommen.
    // Ich empfehle: leer = "keine required definiert" => Good (oder bail).
    // Für jetzt: Good (failed bleibt leer).
    for req in &cfg.required_services {
        if is_ignored_service(req, cfg) {
            continue;
        }

        match services.get(req) {
            Some(status) if status == STARTED => {}
            Some(status) => failed.push(FailedService {
                name: req.clone(),
                status: status.clone(),
            }),
            None => failed.push(FailedService {
                name: req.clone(),
                status: "missing".to_string(),
            }),
        }
    }

    failed
}

fn is_ignored_service(name: &str, cfg: &HealthConfig) -> bool {
    cfg.ignore_exact.iter().any(|s| s == name)
        || cfg.ignore_prefixes.iter().any(|p| name.starts_with(p))
}



pub fn parse_services_map(stdout: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for raw_line in stdout.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with("Runlevel:") || line.starts_with("Dynamic Runlevel:") {
            continue;
        }

        let mut it = line.split_whitespace();
        let Some(svc_name) = it.next() else { continue; };

        let _maybe_bracket = it.next(); // meistens "["
        let Some(status_word) = it.next() else { continue; };

        map.insert(svc_name.to_string(), status_word.to_string());
    }

    map
}

pub fn check_openrc_and_mark() -> Result<()> {
    log::info!("Checking OpenRC services in runlevel 'default'…");

    let output = Command::new("rc-status")
        .args(["--nocolor", "default"])
        .output()
        .context("failed to execute `rc-status --nocolor default`")?;

    if !output.status.success() {
        anyhow::bail!("`rc-status` exited with {}", output.status);
    }

    let stdout =
    String::from_utf8(output.stdout).context("`rc-status` output was not valid UTF-8")?;
    let cfg = HealthConfig::default();
    match decide_health(&stdout, &cfg) {
        HealthDecision::Good => {
            log::info!("All relevant services in runlevel 'default' are started – marking GOOD");
            crate::rauc::mark_good()?;
            Ok(())
        }
        HealthDecision::Bad(failed_services) => {
            log::error!("Some relevant services in runlevel 'default' are NOT started:");
            for svc in &failed_services {
                log::error!("  - {} ({})", svc.name, svc.status);
            }
            log::error!("Marking slot BAD");
            crate::rauc::mark_bad()?;
            anyhow::bail!("OpenRC health check failed");
        }
    }
}