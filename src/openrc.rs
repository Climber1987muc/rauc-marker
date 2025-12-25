use anyhow::{Context, Result};
use clap::Parser;
use std::process::Command;

const STARTED: &str = "started";

// Dienste, die wir bewusst ignorieren (One-Shot, Konsolen, etc.)
const IGNORE_EXACT: &[&str] = &["time-first-boot", "local"];
const IGNORE_PREFIXES: &[&str] = &["getty."];

fn is_ignored_service(name: &str) -> bool {
    IGNORE_EXACT.contains(&name) || IGNORE_PREFIXES.iter().any(|p| name.starts_with(p))
}

fn collect_failed_services(stdout: &str) -> Vec<FailedService> {
    let mut failed_services = Vec::new();

    for raw_line in stdout.lines() {
        let line = raw_line.trim();

        if line.is_empty() {
            continue;
        }
        if line.starts_with("Runlevel:") || line.starts_with("Dynamic Runlevel:") {
            continue;
        }

        let mut it = line.split_whitespace();
        let Some(svc_name) = it.next() else {
            continue;
        };

        let _maybe_bracket = it.next(); // meistens "["

        let Some(status_word) = it.next() else {
            continue;
        };

        if is_ignored_service(svc_name) {
            continue;
        }

        if status_word != STARTED {
            failed_services.push(FailedService {
                name: svc_name.to_string(),
                status: status_word.to_string(),
            });
        }
    }

    failed_services
}


fn check_openrc_and_mark() -> Result<()> {
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
    
    let failed_services = collect_failed_services(&stdout);

    if failed_services.is_empty() {
        log::info!("All relevant services in runlevel 'default' are started – marking GOOD");
        rauc::mark_good()?;
        Ok(())
    } else {
        log::error!("Some relevant services in runlevel 'default' are NOT started:");
        for svc in &failed_services {
            log::error!("  - {} ({})", svc.name, svc.status);
        }

        log::error!("Marking slot BAD");
        rauc::mark_bad()?;

        // Exitcode != 0, damit OpenRC dein reboot auslöst
        anyhow::bail!("OpenRC health check failed");
    }
}