mod config;
mod cli;

use cli::CLI;
use config::{load_config, Config};

fn main() {
    let cli: CLI = argh::from_env();

    let _config = match cli.config {
        Some(ref path) => load_config(path).unwrap(),
        None => Config::default(),
    };

    println!("{:?}", cli);
}
