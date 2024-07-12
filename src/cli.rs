use anyhow::{ensure, Context};
use argh::FromArgs;
use serde::Deserialize;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

/// Automatically checks combinations of feature flags for a Cargo project.
#[allow(clippy::upper_case_acronyms)]
#[derive(FromArgs, Debug)]
pub struct CLI {
    /// the path to `Cargo.toml`
    #[argh(option, default = "locate_manifest_or_exit()")]
    pub manifest_path: PathBuf,

    /// the path to the config folder
    #[argh(option)]
    pub config: Option<PathBuf>,

    /// the chunk that will be checked
    #[argh(option)]
    pub chunk: Option<usize>,

    /// the total amount of chunks
    #[argh(option)]
    pub total_chunks: Option<usize>,
}

impl CLI {
    /// Verifies that the arguments specified are compatible with each other.
    pub fn verify(self) -> anyhow::Result<Self> {
        // Check that, if chunking is enabled, both flags are specified.
        ensure!(
            !(self.chunk.is_some() ^ self.total_chunks.is_some()),
            "`--chunk` and `--total-chunks` require each other. Both or neither must be specified."
        );

        // Check that chunk < total_chunks.
        if let (Some(chunk), Some(total_chunks)) = (self.chunk, self.total_chunks) {
            ensure!(chunk < total_chunks, "Chunk must be within range [0..total_chunks), but is is {chunk} which is >= {total_chunks}.");
        }

        Ok(self)
    }
}

/// Represents the output of `cargo-locate-project`.
#[derive(Deserialize, Debug)]
struct ProjectLocation {
    pub root: PathBuf,
}

/// A variant of [`locate_manifest()`] that pretty-prints errors and calls [`std::process::exit()`].
///
/// This is meant to simulate the behavior of `fn main() -> anyhow::Result`, since [`argh`] cannot
/// easily handle errors.
fn locate_manifest_or_exit() -> PathBuf {
    match locate_manifest() {
        Ok(path) => path,
        Err(e) => {
            let e = e.context("Make sure you are in a Cargo workspace, or manually specify `--manifest-path`.")
                .context("Failed to locate Cargo.toml.");

            eprintln!("{e:?}");

            std::process::exit(1);
        }
    }
}

/// Returns the path of the current project's `Cargo.toml` file, using `cargo-locate-project`.
fn locate_manifest() -> anyhow::Result<PathBuf> {
    let output = Command::new("cargo")
        .stderr(Stdio::inherit()) // Print errors directly to terminal.
        .arg("locate-project")
        .args(["--message-format", "json"]) // Output JSON format.
        .args(["--color", "never"]) // Do not output ANSI escape codes.
        .output()
        .context("Could not spawn `cargo-locate-project` process.")?;

    ensure!(
        output.status.success(),
        "`cargo-locate-project` exited with a non-zero exit code."
    );

    let location: ProjectLocation = serde_json::from_slice(&output.stdout)
        .context("Could not parse JSON output from `cargo-locate-project`.")?;

    Ok(location.root)
}
