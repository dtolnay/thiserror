use std::env;
use std::ffi::OsString;
use std::path::Path;
use std::process::{self, Command, Stdio};

fn main() {
    println!("cargo:rerun-if-env-changed=RUSTC_BOOTSTRAP");

    if compile_probe() {
        println!("cargo:rustc-cfg=error_generic_member_access");
    }
}

fn compile_probe() -> bool {
    if env::var_os("RUSTC_STAGE").is_some() {
        // We are running inside rustc bootstrap. This is a highly non-standard
        // environment with issues such as:
        //
        //     https://github.com/rust-lang/cargo/issues/11138
        //     https://github.com/rust-lang/rust/issues/114839
        //
        // Let's just not use nightly features here.
        return false;
    }

    let rustc = cargo_env_var("RUSTC");
    let out_dir = cargo_env_var("OUT_DIR");
    let probefile = Path::new("build").join("probe.rs");

    // Make sure to pick up Cargo rustc configuration.
    let mut cmd = if let Some(wrapper) = env::var_os("RUSTC_WRAPPER") {
        let mut cmd = Command::new(wrapper);
        // The wrapper's first argument is supposed to be the path to rustc.
        cmd.arg(rustc);
        cmd
    } else {
        Command::new(rustc)
    };

    cmd.stderr(Stdio::null())
        .arg("--edition=2018")
        .arg("--crate-name=thiserror")
        .arg("--crate-type=lib")
        .arg("--emit=metadata")
        .arg("--out-dir")
        .arg(out_dir)
        .arg(probefile);

    if let Some(target) = env::var_os("TARGET") {
        cmd.arg("--target").arg(target);
    }

    // If Cargo wants to set RUSTFLAGS, use that.
    if let Ok(rustflags) = env::var("CARGO_ENCODED_RUSTFLAGS") {
        if !rustflags.is_empty() {
            for arg in rustflags.split('\x1f') {
                cmd.arg(arg);
            }
        }
    }

    match cmd.status() {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

fn cargo_env_var(key: &str) -> OsString {
    env::var_os(key).unwrap_or_else(|| {
        eprintln!(
            "Environment variable ${} is not set during execution of build script",
            key,
        );
        process::exit(1);
    })
}
