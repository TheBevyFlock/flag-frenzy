use argh::FromArgs;
use serde::Deserialize;
use std::{
    io,
    path::PathBuf,
    process::{Command, Stdio},
};

/// Automatically checks combinations of feature flags for a Cargo project.
#[allow(clippy::upper_case_acronyms)]
#[derive(FromArgs, Debug)]
pub struct CLI {
    /// the path to `Cargo.toml`
    #[argh(
        option,
        default = "locate_manifest().expect(\"Failed to find Cargo.toml.\")"
    )]
    pub manifest_path: PathBuf,

    /// the path to the config folder
    #[argh(option)]
    pub config: Option<PathBuf>,
}

/// Represents the output of `cargo-locate-project`.
#[derive(Deserialize, Debug)]
struct ProjectLocation {
    pub root: PathBuf,
}

/// Returns the path of the current project's `Cargo.toml` file, using `cargo-locate-project`.
fn locate_manifest() -> io::Result<PathBuf> {
    let output = Command::new("cargo")
        .stderr(Stdio::inherit()) // Print errors directly to terminal.
        .arg("locate-project")
        .args(["--message-format", "json"]) // Output JSON format.
        .args(["--color", "never"]) // Do not output ANSI escape codes.
        .output()?;

    if output.status.success() {
        let location: ProjectLocation =
            serde_json::from_slice(&output.stdout).map_err(io::Error::other)?;
        Ok(location.root)
    } else {
        Err(io::Error::other(
            "`cargo-locate-project` exited with a non-zero exit code.",
        ))
    }
}
