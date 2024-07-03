use super::PackageConfig;
use std::collections::HashMap;

/// A map of all package configuration.
#[derive(Default, Debug)]
pub struct Config {
    pub packages: HashMap<String, PackageConfig>,
}
