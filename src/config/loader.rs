use super::Config;
use crate::config::PackageConfig;
use anyhow::{ensure, Context};
use std::{collections::HashMap, fs, path::Path};

/// Loads all config in a given folder.
///
/// The folder must only contain TOML files following the pattern of `package_name.toml`.
pub fn load_config(folder: &Path) -> anyhow::Result<Config> {
    let mut packages = HashMap::new();

    for file in fs::read_dir(folder)? {
        let file = file?;
        let path = file.path();
        let mut name = file.file_name().to_string_lossy().into_owned();

        ensure!(
            name.ends_with(".toml"),
            "Config file {path:?} must be TOML (end in \".toml\")."
        );

        // Remove ".toml" from the name.
        name.truncate(name.len() - 5);

        ensure!(!name.is_empty(), "Config file {path:?} cannot be named \".toml\" because the name determines the affected package.");

        let contents = fs::read_to_string(file.path())
            .with_context(|| format!("Failed to read {path:?} to a string."))?;

        let package_config = toml::from_str::<PackageConfig>(&contents)
            .with_context(|| format!("Failed to parse {path:?} as TOML."))?;

        packages.insert(name, package_config);
    }

    Ok(Config::new(packages))
}
