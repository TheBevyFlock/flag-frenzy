mod cli;
mod combos;
mod config;
mod intern;
mod metadata;

use cli::CLI;
use config::{load_config, Config};
use intern::FeatureStorage;
use metadata::{load_metadata, Package};

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

    for package in metadata.packages {
        let Package { name, features } = package;

        let skip: &[String] = config
            .get(&name)
            .map_or(&[], |config| config.features.skip.as_slice());

        let storage = intern_features(features, skip);
    }
}

/// Interns all features within the given [`Vec<String>`], skipping any provided.
fn intern_features(features: Vec<String>, skip: &[String]) -> FeatureStorage {
    let mut storage = FeatureStorage::with_capacity_and_key(features.len());

        for feature in features {
            if skip.contains(&feature) {
                continue;
            }

        storage.insert(feature);
    }

    storage
}
