use super::Manifest;
use anyhow::{bail, ensure, Context};
use std::{
    ffi::OsStr,
    path::Path,
    process::{Command, Stdio},
};

/// The format version of the output of `cargo-metadata`.
const FORMAT_VERSION: &str = "1";

/// Loads the [`Manifest`] of a `Cargo.toml` at the specified `manifest_path` using
/// `cargo-metadata`.
///
/// This function will load metadata for all packages in the workspace, but it will skip
/// dependencies.
///
/// # Errors
///
/// - If `manifest_path` is not a file.
/// - If a new process could not be spawned.
/// - If `cargo-metadata` returned a non-zero exit code.
/// - If the returned JSON could not be deserialized into [`Manifest`].
pub fn load_manifest(manifest_path: &Path) -> anyhow::Result<Manifest> {
    ensure!(manifest_path.is_file(), "{manifest_path:?} is not a file.");

    let output = Command::new("cargo")
        .arg("metadata")
        .args(["--format-version", FORMAT_VERSION])
        .args([OsStr::new("--manifest-path"), manifest_path.as_os_str()])
        .arg("--no-deps") // We only want the crates in this workspace.
        .args(["--color", "never"]) // Do not output ANSI escape codes.
        .stderr(Stdio::inherit()) // Print errors directly to terminal.
        .output()
        .context("Could not spawn `cargo-metadata` process.")?;

    if !output.status.success() {
        bail!("`cargo-metadata` exited with a non-zero exit code.")
    }

    serde_json::from_slice::<Manifest>(&output.stdout)
        .context("Failed to parse output of `cargo-metadata`.")
}
