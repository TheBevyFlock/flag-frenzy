mod chunk;
mod cli;
mod combos;
mod config;
mod intern;
mod metadata;
mod runner;

use anyhow::{bail, Context};
use cli::CLI;
use combos::{estimate_combos, feature_combos};
use config::{load_config, Config};
use intern::FeatureStorage;
use metadata::{load_metadata, Package};
use runner::check_with_features;

fn main() -> anyhow::Result<()> {
    let cli = argh::from_env::<CLI>()
        .verify()
        .context("Failed to verify CLI flags.")?;

    let config = match cli.config {
        Some(ref path) => {
            load_config(path).with_context(|| format!("Failed to load config from {path:?}."))?
        }
        None => Config::default(),
    };

    let metadata = load_metadata(&cli.manifest_path).context("Failed to load Cargo metadata.")?;

    let mut failures = Vec::new();

    for package in metadata.packages {
        let Package { name, features } = package;
        let config = config.get(&name);
        let storage = intern_features(features, &config.features.skip);

        // The number of features or the max combo size, whichever is smaller.
        let max_k = config.features.max_combo_size;

        let estimated_checks = estimate_combos(storage.len() as u128, max_k.map(|k| k as u128))
            .context("Consider decreasing the max combo size in the config.")
            .with_context(|| format!("Total features: {}, Max combo size: {max_k:?}", storage.len()))
            .with_context(|| format!("Unable to estimate checks required for all feature combinations of package {name}."))?;

        println!("Package {name} with {} features.", storage.len());
        println!("Estimated checks: {}", estimated_checks);

        for combo in feature_combos(&storage, max_k) {
            let mut names = Vec::with_capacity(combo.len());

            for &key in combo.iter() {
                names.push(storage.get(key).unwrap());
            }

            println!("\tChecking: {:?}", names);

            let status = check_with_features(&name, &cli.manifest_path, &combo, &storage)
                .with_context(|| format!("Tried checking package {name}."))?;

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

        bail!("Some packages failed to be checked.");
    }

    Ok(())
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
