use serde::{Deserialize, Deserializer};
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
    #[serde(deserialize_with = "deserialize_feature_map_into_vec")]
    pub features: Vec<String>,
}

/// Deserializes a map [`HashMap<String, Vec<String>>`] into a [`Vec<String>`] of its keys.
///
/// Used by [`Package::features`] to discard the feature dependencies.
fn deserialize_feature_map_into_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let result = <HashMap<String, Vec<String>>>::deserialize(deserializer)?;
    Ok(result.into_keys().collect())
}
