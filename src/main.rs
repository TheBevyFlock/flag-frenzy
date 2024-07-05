mod cli;
mod combos;
mod config;
mod intern;
mod metadata;
mod runner;

use cli::CLI;
use combos::{feature_combos, ncr};
use config::{load_config, Config};
use intern::FeatureStorage;
use metadata::{load_metadata, Package};
use runner::check_with_features;

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

    let mut failures = Vec::new();

    for package in metadata.packages {
        let Package { name, features } = package;
        let config = config.get(&name);
        let storage = intern_features(features, &config.features.skip);

        // The number of features or the max combo size, whichever is smaller.
        let max_k = storage
            .len()
            .min(config.features.max_combo_size.unwrap_or(usize::MAX));
        let estimated_checks: u64 = (0..=max_k)
            .map(|k| ncr(storage.len() as u64, k as u64))
            .sum();

        println!("Package {name} with {} features.", storage.len());
        println!("Estimated checks: {}", estimated_checks);

        for combo in feature_combos(&storage, max_k) {
            let mut names = Vec::with_capacity(combo.len());

            for &key in combo.iter() {
                names.push(storage.get(key).unwrap());
            }

            println!("\tChecking: {:?}", names);

            let status = check_with_features(&name, &cli.manifest_path, &combo, &storage);

            if !status.success() {
                failures.push(format!(
                    "Failed checking package {name} with features {names:?}"
                ));
            }
        }
    }

    if !failures.is_empty() {
        eprintln!("Failure report:");

        for failure in failures {
            eprintln!("\t{failure}");
        }
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
