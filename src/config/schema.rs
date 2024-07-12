use serde::{de::Error, Deserialize, Deserializer};

/// Represents the configuration for a specific package.
#[derive(Deserialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct PackageConfig {
    pub features: PackageConfigFeatures,
}

/// Represents feature-specific configuration for a package.
#[derive(Deserialize, Default, Debug)]
#[serde(deny_unknown_fields, default)]
pub struct PackageConfigFeatures {
    /// A list of feature sets that must be enabled.
    ///
    /// A feature set can either be a single string, for which that feature will always be enabled,
    /// or an array of strings, for which combinations will be created.
    ///
    /// ```toml
    /// [features]
    /// required = [
    ///     "foo",
    ///     ["bar", "baz"],
    /// ]
    /// ```
    ///
    /// In this scenario, two combinations of features will be created: `["foo", "bar"]` and
    /// `["foo", "baz"]`.
    pub required: Vec<RequiredFeature>,

    /// A list of feature sets that are incompatible with each other.
    ///
    /// ```toml
    /// [features]
    /// incompatible = [
    ///     ["foo", "baz"],
    /// ]
    /// ```
    ///
    /// In this scenario if, a combination will be skipped if it contains _both_ `foo` and `baz`.
    ///
    /// If you need to always skip a feature, see [`skip`](Self::skip).
    ///
    /// # Panics
    ///
    /// If an individual feature set contains less than 2 features.
    #[serde(deserialize_with = "deserialize_incompatible_features")]
    pub incompatible: Vec<Vec<String>>,

    /// A list of features that will always be skipped.
    ///
    /// ```toml
    /// [features]
    /// skip = ["bar"]
    /// ```
    ///
    /// In this scenario, any combination containing `bar` will be skipped.
    ///
    /// To conditionally skip features based on other features, see
    /// [`incompatible`](Self::incompatible).
    pub skip: Vec<String>,

    /// The limit on the size of combinations tested.
    ///
    /// Due to its nature, adding new features makes the total amount of combinations scale
    /// quadratically. By setting the maximum combination size, it reduces the amount of
    /// combinations tested while still catching most issues.
    ///
    /// ```toml
    /// [features]
    /// max_combo_size = 4
    /// ```
    pub max_combo_size: Option<usize>,

    #[serde(default)]
    pub skip_optional_deps: bool,
}

/// An untagged enum that represents singular or multiple required features.
///
/// See [`PackageConfigFeatures::required`].
#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum RequiredFeature {
    One(String),
    More(Vec<String>),
}

/// Used when deserializing [`PackageConfigFeatures::incompatible`] to ensure that at least 2
/// features are in each feature set.
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

    Ok(result)
}
