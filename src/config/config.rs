use super::PackageConfig;
use std::collections::HashMap;

/// An immutable map of all package configuration.
///
/// The key is the name of the crate, with the value being its corresponding [`PackageConfig`].
#[derive(Default, Debug)]
pub struct Config {
    packages: HashMap<String, PackageConfig>,
    global: PackageConfig,
}

impl Config {
    /// Creates a new [`Config`] from a map of package names to their [`PackageConfig`]s.
    ///
    /// If `global` is a key within `packages`, it will be removed and override the default values
    /// for all other packages.
    pub fn new(mut packages: HashMap<String, PackageConfig>) -> Self {
        let global = packages.remove("global").unwrap_or_default();

        // Apply global's values as defaults to all other configs.
        for PackageConfig { features: package } in packages.values_mut() {
            // Override any defaults. This must be kept up-to-date with `PackageConfigFeatures`.

            if package.required.is_empty() {
                package.required.clone_from(&global.features.required);
            }

            if package.incompatible.iter().all(|set| set.is_empty()) {
                package.incompatible.clone_from(&global.features.incompatible);
            }

            if package.skip.is_empty() {
                package.skip.clone_from(&global.features.skip);
            }

            if package.max_combo_size.is_none() {
                package.max_combo_size.clone_from(&global.features.max_combo_size);
            }
        }

        Config { packages, global }
    }

    /// Returns the configuration for a specific package.
    /// 
    /// If no configuration is found under the name, it will return the global config.
    pub fn get(&self, name: &str) -> &PackageConfig {
        self.packages.get(name).unwrap_or(&self.global)
    }
}
