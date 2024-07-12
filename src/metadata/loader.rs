use super::Metadata;
use anyhow::{bail, ensure, Context};
use std::{
    ffi::OsStr,
    path::Path,
    process::{Command, Stdio},
};

/// The format version of the output of `cargo-metadata`.
const FORMAT_VERSION: &str = "1";

/// Loads the metadata of a `Cargo.toml` at the specified `manifest_path` using `cargo-metadata`.
///
/// # Panics
///
/// If `manifest_path` is not a file.
pub fn load_metadata(manifest_path: &Path) -> anyhow::Result<Metadata> {
    ensure!(manifest_path.is_file(), "{manifest_path:?} is not a file.");

    let output = Command::new("cargo")
        .stderr(Stdio::inherit()) // Print errors directly to terminal.
        .arg("metadata")
        .args([OsStr::new("--manifest-path"), manifest_path.as_os_str()])
        .args(["--format-version", FORMAT_VERSION])
        .arg("--no-deps") // We only want the crates in this workspace.
        .args(["--color", "never"]) // Do not output ANSI escape codes.
        .output()
        .context("Could not spawn `cargo-metadata` process.")?;

    if !output.status.success() {
        bail!("`cargo-metadata` exited with a non-zero exit code.")
    }

    serde_json::from_slice::<Metadata>(&output.stdout)
        .context("Failed to parse output of `cargo-metadata`.")
}
