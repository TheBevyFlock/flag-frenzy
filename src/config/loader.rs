use crate::config::PackageConfig;
use std::{collections::HashMap, fs, io, path::Path};

use super::Config;

/// Loads all config in a given folder.
pub fn load_config(folder: &Path) -> io::Result<Config> {
    assert!(folder.is_dir());

    let mut packages = HashMap::new();

    for file in fs::read_dir(folder)? {
        let file = file?;

        let contents = fs::read_to_string(file.path())?;
        let package_config =
            toml::from_str::<PackageConfig>(&contents).map_err(io::Error::other)?;

        let file_name = file
            .file_name()
            .to_string_lossy()
            .strip_suffix(".toml")
            .ok_or(io::Error::other(
                "Non-TOML file found within config folder.",
            ))?
            .to_string();

        packages.insert(file_name, package_config);
    }

    Ok(Config { packages })
}
