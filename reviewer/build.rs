use anyhow::{Context, Result};
use chrono::prelude::*;
use std::env;
use std::process::Command;
use tera::Tera;

fn main() -> Result<()> {
    let git_hash = get_git_hash().context("failed to get git hash")?;
    let build_date = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let build_profile = env::var("PROFILE").unwrap_or("(unknown)".to_owned());
    let rustc_version = get_rustc_version().context("failed to get rustc version")?;
    let rustc_host = env::var("HOST").unwrap_or("(unknwon)".to_owned());
    let rustc_target = env::var("TARGET").unwrap_or("(unknwon)".to_owned());

    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
    println!("cargo:rustc-env=BUILD_DATE={}", build_date);
    println!("cargo:rustc-env=BUILD_PROFILE={}", build_profile);
    println!("cargo:rustc-env=RUSTC_VERSION={}", rustc_version);
    println!("cargo:rustc-env=RUSTC_HOST={}", rustc_host);
    println!("cargo:rustc-env=RUSTC_TARGET={}", rustc_target);

    if build_profile == "debug" {
        // check the templates at compile time.
        Tera::new("templates/**/*.html").context("failed to parse templates")?;
    }

    Ok(())
}

fn get_git_hash() -> Result<String> {
    let output = Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()?;

    if !output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        return Ok("???".to_owned());
    }

    let stdout = String::from_utf8(output.stdout)?;
    let git_hash = match stdout.trim() {
        "" => "???",
        v => v,
    };

    Ok(git_hash.to_owned())
}

fn get_rustc_version() -> Result<String> {
    let rustc = env::var("RUSTC").unwrap_or("rustc".to_owned());

    let output = Command::new(rustc).args(&["--version"]).output()?;

    if !output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        return Ok("rustc ??? (??? ???)".to_owned());
    }

    let stdout = String::from_utf8(output.stdout)?;
    let rustc_version = match stdout.trim() {
        "" => "rustc ??? (??? ???)",
        v => v,
    };

    Ok(rustc_version.to_owned())
}
