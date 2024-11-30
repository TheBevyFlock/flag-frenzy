mod ansi;
mod chunk;
mod cli;
mod combos;
mod config;
mod intern;
mod manifest;
mod runner;

use ansi::*;
use anyhow::{bail, Context};
use chunk::select_chunk;
use cli::CLI;
use combos::{estimate_combos, feature_combos};
use config::{load_config, WorkspaceConfig};
use intern::intern_features;
use manifest::{load_manifest, Manifest, Package};
use runner::check_with_features;
use std::path::Path;
#[cfg(test)]
mod tests;

fn main() -> anyhow::Result<()> {
    let cli = CLI::from_env().context("Failed to verify CLI flags.")?;

    let config_path = match cli.config {
        Some(ref path) => path,
        None => Path::new("config"),
    };

    let config = if config_path.is_dir() {
        load_config(config_path)
            .with_context(|| format!("Failed to load config from {config_path:?}."))?
    } else {
        eprintln!("No config folder found, using default config.");
        WorkspaceConfig::default()
    };

    let Color {
        reset,
        bold,
        dim,
        info,
        success,
        error,
    } = Color::from_color_choice(cli.color);

    let manifest = load_manifest(&cli.manifest_path).context("Failed to load Cargo manifest.")?;

    let packages =
        process_packages(manifest, &cli, &config).context("Failure while processing packages.")?;

    let mut failures = Vec::new();

    for package in packages {
        let Package { name, features } = package;
        let package_config = config.get(&name);
        let storage = intern_features(features, package_config);

        // The number of features or the max combo size, whichever is smaller.
        let max_k = package_config.max_combo_size();

        let estimated_checks = estimate_combos(storage.len() as u128, max_k.map(|k| k as u128))
            .context("Consider decreasing the max combo size in the config.")
            .with_context(|| format!("Total features: {}, Max combo size: {max_k:?}", storage.len()))
            .with_context(|| format!("Unable to estimate checks required for all feature combinations of package {name}."))?;

        println!(
            "{bold}Package {info}{name}{reset}{bold} with {info}{}{reset}{bold} features.{reset}",
            storage.len()
        );
        println!("{bold}Estimated checks: {info}{estimated_checks}{reset}");

        let mut actual_checks = 0;
        for combo in feature_combos(&storage, package_config) {
            actual_checks += 1;
            let mut features = Vec::with_capacity(combo.len());

            for &key in combo.iter() {
                features.push(storage.get(key).unwrap());
            }

            features.sort_unstable();

            println!("\t{dim}Checking:{reset} {info}{:?}{reset}", features);

            if cli.dry_run {
                continue;
            }
            let status = check_with_features(&name, &cli.manifest_path, &combo, &storage)
                .with_context(|| format!("Tried checking package {name}."))?;

            if !status.success() {
                failures.push(CheckFailure {
                    package: name.clone(),
                    features: features.into_iter().map(str::to_string).collect(),
                });
            }
        }
        println!("{bold}Actual checks: {info}{actual_checks}{reset}");
    }

    if !failures.is_empty() {
        eprintln!("{error}{bold}Failure report:{reset}");

        for CheckFailure { package, features } in failures {
            eprintln!("\t{error}Failed checking package {bold}{package}{reset} {error}with features{reset} {features:?}.");
        }

        bail!("Some packages failed to be checked.");
    }

    if cli.dry_run {
        println!("{info}{bold}Dry run completed, no checks were run.{reset}");
    } else {
        println!("{success}{bold}Feature combination checks successful! Congrats :){reset}");
    }

    Ok(())
}

/// Processes the packages in a [`Manifest`] and returns them in a [`Vec`].
///
/// Specifically, this:
///
/// - Returns a single package if `--package` is specified in the CLI.
/// - Sorts the packages by their name.
/// - Filters packages into chunks if enabled.
pub(crate) fn process_packages(
    manifest: Manifest,
    cli: &CLI,
    config: &WorkspaceConfig,
) -> anyhow::Result<Vec<Package>> {
    let mut packages = manifest.packages;

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
        packages = select_chunk(total_chunks, chunk, packages, config);
    }

    Ok(packages)
}

struct CheckFailure {
    pub package: String,
    pub features: Vec<String>,
}
