use serde::Deserialize;
use std::collections::HashMap;

/// A subset of the metadata returned by `cargo-metadata` that's required for `flag-frenzy`.
#[derive(Deserialize, Debug)]
pub struct Manifest {
    /// Contains a list of packages in this workspace.
    ///
    /// If [`load_manifest()`](super::load_manifest) is used, this will not contain any external
    /// dependencies.
    pub packages: Vec<Package>,
}

/// Represents a single package.
#[derive(Deserialize, Debug)]
pub struct Package {
    /// The name of the crate.
    pub name: String,
    /// A list of all features in a crate.
    pub features: HashMap<String, Vec<String>>,
}
