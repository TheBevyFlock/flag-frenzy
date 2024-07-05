use super::PackageConfig;
use std::{collections::HashMap, ops::Deref};

/// A map of all package configuration.
///
/// The key is the name of the crate, with the value being its corresponding [`PackageConfig`].
#[derive(Default, Debug)]
pub struct Config {
    packages: HashMap<String, PackageConfig>,
}

impl Config {
    pub fn new(packages: HashMap<String, PackageConfig>) -> Self {
        Config { packages }
    }
}

impl Deref for Config {
    type Target = HashMap<String, PackageConfig>;

    fn deref(&self) -> &Self::Target {
        &self.packages
    }
}
