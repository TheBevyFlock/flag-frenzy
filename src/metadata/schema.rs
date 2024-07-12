use serde::Deserialize;
use std::collections::HashMap;

/// A subset of the metadata returned by `cargo-metadata` that's required for this application.
#[derive(Deserialize, Debug)]
pub struct Metadata {
    /// Contains a list of packages in this workspace.
    ///
    /// If [`load_metadata()`](super::load_metadata) is used, this will not contain any
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
