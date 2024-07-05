mod cli;
mod combos;
mod config;
mod metadata;

use cli::CLI;
use config::{load_config, Config};
use metadata::load_metadata;

fn main() {
    let cli: CLI = argh::from_env();

    #[cfg(debug_assertions)]
    println!("{:?}", cli);

    let config = match cli.config {
        Some(ref path) => load_config(path).expect("Failed to load config."),
        None => Config::default(),
    };

    #[cfg(debug_assertions)]
    println!("{:?}", config);

    let metadata = load_metadata(&cli.manifest_path).expect("Failed to parse Cargo metadata.");

    #[cfg(debug_assertions)]
    println!("{:?}", metadata);
}
