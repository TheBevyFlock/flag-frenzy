use crate::config::PackageConfig;
use std::{
    collections::HashMap,
    hash::{BuildHasher, RandomState},
};

/// A cheap key to a feature in [`FeatureStorage`].
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FeatureKey(u64);

/// A container that interns [`String`]s for feature flags, and returns [`FeatureKey`] for easy
/// access.
///
/// This is internally a primitive hash map built on binary search. It ignores hash collisions, and
/// should generally not be used for any other purpose.
pub struct FeatureStorage {
    inner: Vec<(u64, String)>,
    build_hasher: RandomState,
}

impl FeatureStorage {
    /// Creates a new [`FeatureStorage`] with a pre-allocated capacity.
    ///
    /// See [`Vec::with_capacity()`] for a greater explanation on what this does.
    pub fn with_capacity(capacity: usize) -> Self {
        FeatureStorage {
            inner: Vec::with_capacity(capacity),
            build_hasher: RandomState::new(),
        }
    }

    /// Retrieves a feature name from a key.
    ///
    /// This will return [`None`] if nothing is found.
    pub fn get(&self, key: FeatureKey) -> Option<&str> {
        match self.inner.binary_search_by_key(&key.0, |(h, _)| *h) {
            Ok(i) => Some(&self.inner[i].1),
            Err(_) => None,
        }
    }

    /// Returns how many features are in storage.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns an iterator over all [`FeatureKey`]s in this map.
    pub fn keys(&self) -> impl Iterator<Item = FeatureKey> + '_ {
        self.inner.iter().map(|(h, _)| FeatureKey(*h))
    }

    /// Inserts a feature into storage, returning its key.
    pub fn insert(&mut self, feature: String) -> FeatureKey {
        let hash = self.create_key(&feature).0;

        match self.inner.binary_search_by_key(&hash, |(h, _)| *h) {
            // Feature already exists in storage, do nothing.
            Ok(i) => debug_assert_eq!(self.inner[i].1, feature, "Congrats, you found a hash collision! This is incredibly rare, and likely won't happen if you re-run the program because the initial state of the hasher is determined by the OS. Cool!"),
            // Feature does not exist, add it!
            Err(i) => self.inner.insert(i, (hash, feature)),
        }

        FeatureKey(hash)
    }

    /// Creates a key for a given string.
    ///
    /// Note that this does not actually insert the string into the map. See [`Self::insert()`] for
    /// this behavior.
    pub fn create_key(&self, s: &str) -> FeatureKey {
        FeatureKey(self.build_hasher.hash_one(s))
    }
}

/// Interns all features within the given [`Vec<String>`].
///
/// This skips features specified in passed [`PackageConfig`], and additionally optional
/// dependencies if enabled.
pub fn intern_features(
    features: HashMap<String, Vec<String>>,
    PackageConfig { features: config }: &PackageConfig,
) -> FeatureStorage {
    /// Returns true if a feature is likely an optional dependency.
    ///
    /// This is done by detecting if the feature dependencies solely contains a crate of the same name.
    /// If `feature` was `"foo"` and deps was `["dep:foo"]`, for example, then it is likely an optional
    /// dependency.
    fn is_optional_dep(feature: &str, deps: &[String]) -> bool {
        deps == [format!("dep:{feature}")]
    }

    let mut storage = FeatureStorage::with_capacity(features.len());

    for (feature, deps) in features {
        // If the feature should be skipped, or is an optional dependency, don't add it to storage.
        if config.skip.contains(&feature)
            || (config.skip_optional_deps && is_optional_dep(&feature, &deps))
        {
            continue;
        }

        storage.insert(feature);
    }

    storage
}
