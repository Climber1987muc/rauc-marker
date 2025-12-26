// src/bin/build_and_copy.rs
//
// Build the project in release mode and copy artifacts into a staging "install" tree.
//
// Usage examples:
//   cargo run --bin build_and_copy
//   cargo run --bin build_and_copy -- --dest target/install
//
// Notes:
// - This is great for local/testing workflows.
// - For Yocto, prefer doing the copy/install in the recipe's do_install() step.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

fn ensure_parent(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn run_cargo_release_build() -> Result<ExitStatus, String> {
    let status = Command::new("cargo")
        .args(["build", "--release"])
        .status()
        .map_err(|e| format!("failed to run `cargo build --release`: {e}"))?;

    Ok(status)
}

fn copy_file(src: &Path, dst: &Path, what: &str) -> Result<(), String> {
    ensure_parent(dst).map_err(|e| format!("failed to create parent dirs for {what}: {e}"))?;
    fs::copy(src, dst).map_err(|e| format!("failed to copy {what} from {src:?} to {dst:?}: {e}"))?;
    Ok(())
}

/// Very small CLI parser:
///   --dest <PATH>   staging root (default: target/install)
fn parse_dest_root() -> Result<PathBuf, String> {
    let mut args = env::args().skip(1);
    let mut dest_root: PathBuf = PathBuf::from("target/install");

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--dest" => {
                let p = args
                    .next()
                    .ok_or_else(|| "missing value after --dest".to_string())?;
                dest_root = PathBuf::from(p);
            }
            "--help" | "-h" => {
                println!(
                    "build_and_copy\n\n\
                     Builds release and stages artifacts.\n\n\
                     Options:\n\
                     \t--dest <PATH>\tStaging root (default: target/install)\n"
                );
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument: {other} (try --help)")),
        }
    }

    Ok(dest_root)
}

fn main() {
    if let Err(e) = real_main() {
        eprintln!("ERROR: {e}");
        std::process::exit(1);
    }
}

fn real_main() -> Result<(), String> {
    let dest_root = parse_dest_root()?;

    // 1) Build release
    let status = run_cargo_release_build()?;
    if !status.success() {
        return Err(format!("`cargo build --release` failed with status: {status}"));
    }

    // 2) Stage files
    let src_conf = Path::new("config/rauc-health.toml");
    let dst_conf = dest_root.join("etc/rauc/rauc-health.toml");

    let src_bin = Path::new("target/release/rauc-health");
    let dst_bin = dest_root.join("bin/rauc-health");

    if !src_conf.exists() {
        return Err(format!("config file not found: {:?}", src_conf));
    }
    if !src_bin.exists() {
        return Err(format!(
            "binary not found after build: {:?}\n\
             (check your package/bin name; expected target/release/rauc-health)",
            src_bin
        ));
    }

    copy_file(src_conf, &dst_conf, "config")?;
    copy_file(src_bin, &dst_bin, "binary")?;

    println!("Staged artifacts to {:?}", dest_root);
    println!("  {:?} -> {:?}", src_conf, dst_conf);
    println!("  {:?} -> {:?}", src_bin, dst_bin);

    Ok(())
}
