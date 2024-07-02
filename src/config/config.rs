use std::collections::HashMap;
use super::PackageConfig;

/// A map of all package configuration.
#[derive(Debug)]
pub struct Config {
    pub packages: HashMap<String, PackageConfig>,
}
