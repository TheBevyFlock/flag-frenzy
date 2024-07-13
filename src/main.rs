mod chunk;
mod cli;
mod combos;
mod config;
mod intern;
mod metadata;
mod runner;

use anyhow::{bail, Context};
use chunk::select_chunk;
use cli::CLI;
use combos::{estimate_combos, feature_combos};
use config::{load_config, Config, PackageConfig};
use intern::FeatureStorage;
use metadata::{load_metadata, Metadata, Package};
use runner::check_with_features;
use std::collections::HashMap;

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

    let packages =
        process_packages(metadata, &cli, &config).context("Failure while processing packages.")?;

    let mut failures = Vec::new();

    for package in packages {
        let Package { name, features } = package;
        let package_config = config.get(&name);
        let storage = intern_features(features, &package_config);

        // The number of features or the max combo size, whichever is smaller.
        let max_k = package_config.features.max_combo_size;

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

/// Processes the packages in a [`Metadata`] and returns them in a [`Vec`].
///
/// Specifically, this:
///
/// - Returns a single package if `--package` is specified in the CLI.
/// - Sorts the packages by their name.
/// - Filters packages into chunks if enabled.
fn process_packages(
    metadata: Metadata,
    cli: &CLI,
    config: &Config,
) -> anyhow::Result<Vec<Package>> {
    let mut packages = metadata.packages;

    // Handle `--package` specifier.
    if let Some(name) = &cli.package {
        let mut package = None;

        // Search for a package with the same name as specified by the CLI.
        for i in 0..packages.len() {
            if &packages[i].name == name {
                package = Some(packages.swap_remove(i));
                break;
            }
        }

        // If a package is found, return it.
        match package {
            Some(package) => return Ok(vec![package]),
            None => bail!("Could not find package {name} specified by `--package`."),
        }
    }

    // Sort packages based on name.
    packages.sort_unstable_by(|a, b| a.name.cmp(&b.name));

    // Filter packages into chunks, if enabled.
    if let (Some(chunk), Some(total_chunks)) = (cli.chunk, cli.total_chunks) {
        packages = select_chunk(total_chunks, chunk, packages, &config);
    }

    Ok(packages)
}

/// Interns all features within the given [`Vec<String>`], skipping any provided.
fn intern_features(
    features: HashMap<String, Vec<String>>,
    PackageConfig { features: config }: &PackageConfig,
) -> FeatureStorage {
    let mut storage = FeatureStorage::with_capacity_and_key(features.len());

    for (feature, deps) in features {
        if config.skip.contains(&feature)
            || (config.skip_optional_deps && is_optional_dep(&feature, &deps))
        {
            continue;
        }

        storage.insert(feature);
    }

    storage
}

fn is_optional_dep(feature: &str, deps: &[String]) -> bool {
    deps == [format!("dep:{feature}")]
}
