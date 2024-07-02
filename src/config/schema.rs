use serde::{de::Error, Deserialize, Deserializer};

/// Represents the configuration for a specific package.
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct PackageConfig {
    pub features: PackageConfigFeatures,
}

/// Represents feature-specific configuration for a package.
#[derive(Deserialize, Default, Debug)]
#[serde(deny_unknown_fields, default)]
pub struct PackageConfigFeatures {
    /// A list of feature sets that must be enabled.
    pub required: Vec<FeatureSet>,

    /// A list of feature sets that are incompatible with each other.
    #[serde(deserialize_with = "deserialize_incompatible_features")]
    pub incompatible: Vec<Vec<String>>,

    /// A list of features that will always be skipped.
    pub skip: Vec<String>,
}

/// Represents a feature set: one or more packages.
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum FeatureSet {
    One(String),
    More(Vec<String>),
}

fn deserialize_incompatible_features<'de, D>(deserializer: D) -> Result<Vec<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let result = <Vec<Vec<String>>>::deserialize(deserializer)?;

    // Check that there are at least 2 features in an incompatible set.
    for feature_set in result.iter() {
        if feature_set.len() < 2 {
            return Err(D::Error::invalid_length(feature_set.len(), &"2 or greater"));
        }
    }

    return Ok(result);
}
