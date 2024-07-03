use super::Metadata;
use std::{
    ffi::OsStr,
    io,
    path::Path,
    process::{Command, Stdio},
};

/// The format version of the output of `cargo-metadata`.
const FORMAT_VERSION: &str = "1";

pub fn load_metadata(manifest_path: &Path) -> io::Result<Metadata> {
    assert!(manifest_path.is_file());

    let output = Command::new("cargo")
        .stderr(Stdio::inherit()) // Print errors directly to terminal.
        .arg("metadata")
        .args([OsStr::new("--manifest-path"), manifest_path.as_os_str()])
        .args(["--format-version", FORMAT_VERSION])
        .arg("--no-deps") // We only want the crates in this workspace.
        .args(["--color", "never"]) // Do not output ANSI escape codes.
        .output()?;

    if output.status.success() {
        serde_json::from_slice(&output.stdout).map_err(io::Error::other)
    } else {
        Err(io::Error::other(
            "`cargo-metadata` exited with a non-zero exit code.",
        ))
    }
}
