mod config;

use std::path::Path;
use config::load_config;

fn main() {
    let config = load_config(Path::new("config")).unwrap();

    println!("Config: {:#?}", config);
}
