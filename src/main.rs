mod cli;
mod config;
mod features;
mod metadata;

use cli::CLI;
use config::{load_config, Config};
use metadata::load_metadata;

fn main() {
    let cli: CLI = argh::from_env();

    let _config = match cli.config {
        Some(ref path) => load_config(path).expect("Failed to load config."),
        None => Config::default(),
    };

    let metadata = load_metadata(&cli.manifest_path).expect("Failed to parse Cargo metadata.");

    println!("{:?}", metadata);
}
