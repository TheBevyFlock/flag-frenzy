use argh::FromArgs;
use std::path::PathBuf;

/// Automatically checks combinations of feature flags for a Cargo project.
#[derive(FromArgs, Debug)]
pub struct CLI {
    /// the path to `Cargo.toml`
    #[argh(option, default = "PathBuf::from(\"Cargo.toml\")")]
    pub manifest_path: PathBuf,

    /// package(s) to check
    #[argh(option, short = 'p')]
    pub package: Vec<String>,

    /// the path to the config folder
    #[argh(option)]
    pub config: Option<PathBuf>,
}
