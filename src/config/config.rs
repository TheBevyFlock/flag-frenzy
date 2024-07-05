use super::PackageConfig;
use std::collections::HashMap;

/// A map of all package configuration.
///
/// The key is the name of the crate, with the value being its corresponding [`PackageConfig`].
#[derive(Default, Debug)]
pub struct Config {
    pub packages: HashMap<String, PackageConfig>,
}
