use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::str;

// This code exercises the surface area that we expect of the Error trait's generic member access.
// If the current toolchain is able to compile it, then thiserror is able to offer backtrace
// support via generic member access.
const PROBE: &str = r#"
    #![feature(error_generic_member_access)]

    use std::any::Request;

    #[derive(Debug)]
    struct ErrorWithGenericMemberAccess {
        it: u32,
    }

    impl std::fmt::Display for ErrorWithGenericMemberAccess {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "ErrorWithGenericMemberAccess")
        }
    }

    impl std::error::Error for ErrorWithGenericMemberAccess {
        fn provide<'a>(&'a self, request: &mut Request<'a>) {
            request.provide_value::<u32>(self.it);
        }
    }

    fn _get_u32(e: &dyn std::error::Error) -> Option<u32> {
        e.request_value::<u32>()
    }
"#;

fn main() {
    match compile_probe() {
        Some(status) if status.success() => println!("cargo:rustc-cfg=error_generic_member_access"),
        _ => {},
    }
}

fn compile_probe() -> Option<ExitStatus> {
    let rustc = env::var_os("RUSTC")?;
    let out_dir = env::var_os("OUT_DIR")?;
    let probefile = Path::new(&out_dir).join("probe.rs");
    fs::write(&probefile, PROBE).ok()?;

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
        .arg("--crate-name=thiserror_build")
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

    cmd.status().ok()
}
