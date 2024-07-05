use super::PackageConfig;
use std::{collections::HashMap, ops::Deref};

/// An immutable map of all package configuration.
///
/// The key is the name of the crate, with the value being its corresponding [`PackageConfig`].
#[derive(Default, Debug)]
pub struct Config {
    packages: HashMap<String, PackageConfig>,
}

impl Config {
    /// Creates a new [`Config`] from a map of package names to their [`PackageConfig`]s.
    ///
    /// If `global` is a key within `packages`, it will be removed and override the default values
    /// for all other packages.
    pub fn new(mut packages: HashMap<String, PackageConfig>) -> Self {
        // If `global.toml` exists, apply its defaults to all other configs.
        if let Some(PackageConfig { features: global }) = packages.remove("global") {
            for PackageConfig { features: package } in packages.values_mut() {
                // Override any defaults. This must be kept up-to-date with `PackageConfigFeatures`.

                if package.required.is_empty() {
                    package.required.clone_from(&global.required);
                }

                if package.incompatible.iter().all(|set| set.is_empty()) {
                    package.incompatible.clone_from(&global.incompatible);
                }

                if package.skip.is_empty() {
                    package.skip.clone_from(&global.skip);
                }

                if package.max_combo_size.is_none() {
                    package.max_combo_size.clone_from(&global.max_combo_size);
                }
            }
        }

        Config { packages }
    }
}

impl Deref for Config {
    type Target = HashMap<String, PackageConfig>;

    fn deref(&self) -> &Self::Target {
        &self.packages
    }
}
