//! Feature key internment.
//!
//! All feature names are [interned](https://en.wikipedia.org/wiki/String_interning) in the
//! [`FeatureStorage`] type, which maps [`String`]s to cheap, easily-cloneable [`FeatureKey`]s.

use crate::config::Config;
use std::{
    collections::{BTreeSet, HashMap},
    hash::{BuildHasher, RandomState},
};

/// A cheap key to a feature in [`FeatureStorage`].
///
/// This type is just a single [`u64`], so it can be copied easily without loss in performance.
///
/// Note that keys are not equivalent across [`FeatureStorage`]s. Inserting the same [`String`]
/// into two separate [`FeatureStorage`]s will likely result in two different [`FeatureKey`]s, and
/// using the same [`FeatureKey`] to get the string from two separate [`FeatureStorage`]s will likely result
/// in two different strings (if [`FeatureStorage::get()`] doesn't return [`None`], that is).
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct FeatureKey(u64);

/// A container that interns [`String`]s for feature flags, and returns [`FeatureKey`]s for easy
/// access.
///
/// This is internally a primitive hash map built on binary search. It ignores hash collisions, and
/// should generally not be used for any other purpose.
#[derive(Debug)]
pub struct FeatureStorage {
    /// A list of feature entries. The [`u64`] is the [`FeatureKey`], and the [`String`] is the
    /// associated feature.
    ///
    /// This must be sorted based on the [`u64`], since lookups use binary search.
    inner: Vec<(u64, String, BTreeSet<FeatureKey>)>,
    /// The hashing state, used to calculate the hash (and thus the [`FeatureKey`]) of features.
    ///
    /// The hash of two identical values using the same [`RandomState`] will result in the same
    /// hash, but using two separate [`RandomState`]s may result in different hashes.
    ///
    /// As a result, inserting a feature into two separate [`FeatureStorage`]s may not result in
    /// the same [`FeatureKey`].
    build_hasher: RandomState,
}

impl FeatureStorage {
    /// Shortcut function to create a new [`FeatureStorage`].
    ///
    /// As this is not used by the main application, it is only available during testing.
    #[cfg(test)]
    pub fn new() -> Self {
        FeatureStorage {
            inner: Vec::new(),
            build_hasher: RandomState::new(),
        }
    }

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
    #[must_use]
    pub fn get(&self, key: FeatureKey) -> Option<&str> {
        match self.inner.binary_search_by_key(&key.0, |(h, ..)| *h) {
            Ok(i) => Some(&self.inner[i].1),
            Err(_) => None,
        }
    }

    /// Returns the set of all direct and indirect dependencies of the feature.
    ///
    /// This will return [`None`] if nothing is found.
    #[must_use]
    pub fn get_dependencies(&self, key: FeatureKey) -> Option<&BTreeSet<FeatureKey>> {
        match self.inner.binary_search_by_key(&key.0, |(h, ..)| *h) {
            Ok(i) => Some(&self.inner[i].2),
            Err(_) => None,
        }
    }

    /// Returns true if `dependency_key` represents a direct or indirect dependency of `key`.
    ///
    /// This will return `false` if nothing is found.
    #[must_use]
    pub fn is_dependency(&self, key: FeatureKey, dependency_key: FeatureKey) -> bool {
        self.get_dependencies(key)
            .map(|dependencies| dependencies.contains(&dependency_key))
            .unwrap_or_default()
    }

    /// Inserts a feature into storage, returning its key.
    pub fn insert(
        &mut self,
        feature: String,
        features_map: &HashMap<String, Vec<String>>,
    ) -> FeatureKey {
        let mut dependencies_keys = BTreeSet::new();
        if let Some(dependencies) = features_map.get(&feature) {
            for dependency in dependencies {
                // we ignore - dependencies which aren't in the list of features,
                // or dependencies that have already been added,
                // or dependencies which aren't other features
                if !features_map.contains_key(dependency)
                    || dependencies_keys.contains(&self.create_key(dependency))
                    || dependency.starts_with("dep:")
                {
                    continue;
                }
                let key = self.insert(dependency.clone(), features_map);
                dependencies_keys.insert(key);

                if let Some(sub_dependencies) = self.get_dependencies(key) {
                    dependencies_keys = dependencies_keys
                        .union(sub_dependencies)
                        .map(|val| *val)
                        .collect();
                }
            }
        }
        let hash = self.create_key(&feature).0;

        match self.inner.binary_search_by_key(&hash, |(h, ..)| *h) {
            // Feature already exists in storage, do nothing.
            Ok(i) => debug_assert_eq!(self.inner[i].1, feature, "Congrats, you found a hash collision! This is incredibly rare, and likely won't happen if you re-run the program because the initial state of the hasher is determined by the OS. Cool!"),
            // Feature does not exist, add it!
            Err(i) => self.inner.insert(i, (hash, feature, dependencies_keys)),
        };

        FeatureKey(hash)
    }

    /// Returns how many features are in storage.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns an iterator over all [`FeatureKey`]s in this map.
    pub fn keys(&self) -> impl Iterator<Item = FeatureKey> + '_ {
        self.inner.iter().map(|(h, ..)| FeatureKey(*h))
    }

    /// Creates a key for a given string.
    ///
    /// Note that this does not actually insert the string into the map. See [`Self::insert()`] for
    /// this behavior.
    #[must_use]
    pub fn create_key(&self, s: &str) -> FeatureKey {
        FeatureKey(self.build_hasher.hash_one(s))
    }
}

/// Interns all features within the given [`Vec<String>`].
///
/// This skips optional dependencies, if enabled in the passed [`Config`].
pub fn intern_features(
    features: HashMap<String, Vec<String>>,
    config: Config<'_>,
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

    // We cache this output, since `skip_optional_deps()` is heavier than a simple lookup.
    let skip_optional_deps = config.skip_optional_deps();

    // remove optional features from the feature map, so that they won't be added in recursive calls to `insert`
    let features: HashMap<_, _> = features
        .into_iter()
        .filter(|(feature, deps)| !skip_optional_deps || !is_optional_dep(feature, deps))
        .collect();

    for (feature, _) in features.iter() {
        storage.insert(feature.clone(), &features);
    }

    storage
}

#[cfg(test)]
mod tests {
    use crate::config::WorkspaceConfig;

    use super::*;

    #[test]
    fn equal_features_are_equal_keys() {
        let storage = FeatureStorage::new();

        const FEATURE: &str = "nyan-cat";

        let (key1, key2) = (storage.create_key(FEATURE), storage.create_key(FEATURE));

        assert_eq!(
            key1, key2,
            "Hashing the same feature with the same storage does not produce the same key."
        );
    }

    #[test]
    fn insert_and_get_feature() {
        let mut storage = FeatureStorage::new();

        let keys: Vec<_> = (0..5)
            .map(|i| storage.insert(i.to_string(), &HashMap::new()))
            .collect();

        for i in 0..5 {
            assert_eq!(
                storage.get(keys[i]).unwrap(),
                i.to_string(),
                "The feature returned by `FeatureStorage::get()` is incorrect."
            );
        }
    }

    #[test]
    fn iter_keys() {
        let mut storage = FeatureStorage::new();

        let mut inserted_keys: Vec<_> = (0..10)
            .map(|i| storage.insert(i.to_string(), &HashMap::new()))
            .map(|FeatureKey(h)| h)
            .collect();

        let mut retrieved_keys: Vec<_> = storage.keys().map(|FeatureKey(h)| h).collect();

        // We sort both lists to ensure that the order is equal, so that they can be compared.
        inserted_keys.sort_unstable();
        retrieved_keys.sort_unstable();

        assert_eq!(inserted_keys, retrieved_keys, "The keys returned by `FeatureStorage::insert()` are not the keys returned by `FeatureStorage::keys()`.");
    }

    #[test]
    fn len() {
        let mut storage = FeatureStorage::new();

        assert_eq!(
            storage.len(),
            0,
            "`FeatureStorage` did not start out empty."
        );

        // Insert 2 unique features.
        storage.insert("hello".to_string(), &HashMap::new());
        storage.insert("goodbye".to_string(), &HashMap::new());

        assert_eq!(
            storage.len(),
            2,
            "`FeatureStorage::len()` did not reflect the amount of features in storage."
        );

        // Insert a duplicate feature.
        storage.insert("hello".to_string(), &HashMap::new());

        assert_eq!(
            storage.len(),
            2,
            "Feature was not de-duplicated by `FeatureStorage::insert()`."
        );
    }

    #[test]
    fn map_sub_dependencies() {
        let mut features_map = HashMap::new();
        features_map.insert("foo".to_string(), Vec::new());
        features_map.insert("bar".to_string(), vec!["foo".to_string()]);
        features_map.insert("foobar".to_string(), vec!["bar".to_string()]);
        features_map.insert("unrelated".to_string(), Vec::new());

        let schema_config = crate::config::schema::Config::default();
        let workspace_config = WorkspaceConfig::new(HashMap::new(), schema_config);
        let storage = intern_features(features_map, workspace_config.get(""));

        let foobar_key = storage.create_key("foobar");
        let foo_key = storage.create_key("foo");
        let bar_key = storage.create_key("bar");
        let unrelated_key = storage.create_key("unrelated");
        assert_eq!(storage.len(), 4);

        assert!(storage.is_dependency(foobar_key, foo_key));
        assert!(storage.is_dependency(foobar_key, bar_key));
        assert!(!storage.is_dependency(foobar_key, unrelated_key));
        assert!(!storage.is_dependency(foobar_key, foobar_key));

        assert!(!storage.is_dependency(foo_key, foo_key));
        assert!(!storage.is_dependency(foo_key, bar_key));
        assert!(!storage.is_dependency(foo_key, foobar_key));
        assert!(!storage.is_dependency(foo_key, unrelated_key));

        assert!(storage.is_dependency(bar_key, foo_key));
        assert!(!storage.is_dependency(bar_key, foobar_key));
        assert!(!storage.is_dependency(bar_key, bar_key));
        assert!(!storage.is_dependency(bar_key, unrelated_key));

        assert!(!storage.is_dependency(unrelated_key, foo_key));
        assert!(!storage.is_dependency(unrelated_key, bar_key));
        assert!(!storage.is_dependency(unrelated_key, foobar_key));
        assert!(!storage.is_dependency(unrelated_key, unrelated_key));
    }

    #[test]
    fn handle_self_named_dependencies() {
        let mut features_map = HashMap::new();
        features_map.insert("foobar".to_string(), vec!["foo".to_string()]);
        features_map.insert("foo".to_string(), vec!["dep:foo".to_string()]);

        let schema_config = crate::config::schema::Config::default();
        let workspace_config = WorkspaceConfig::new(HashMap::new(), schema_config);
        let storage = intern_features(features_map, workspace_config.get(""));

        let foobar_key = storage.create_key("foobar");
        let foo_key = storage.create_key("foo");
        let dep_foo_key = storage.create_key("dep:foo");

        assert_eq!(storage.len(), 2);
        assert_eq!(storage.get(foobar_key), Some("foobar"));
        assert_eq!(storage.get(foo_key), Some("foo"));
        assert_eq!(storage.get(dep_foo_key), None);
        assert!(storage.is_dependency(foobar_key, foo_key));
        assert!(!storage.is_dependency(foo_key, foobar_key));
    }

    #[test]
    fn filter_optional_features() {
        let mut features_map = HashMap::new();
        features_map.insert("foobar".to_string(), vec!["foo".to_string()]);
        features_map.insert("foo".to_string(), vec!["dep:foo".to_string()]);

        let mut schema_config = crate::config::schema::Config::default();
        schema_config.skip_optional_deps = Some(true);
        let workspace_config = WorkspaceConfig::new(HashMap::new(), schema_config);
        let storage = intern_features(features_map, workspace_config.get(""));

        let foobar_key = storage.create_key("foobar");
        let foo_key = storage.create_key("foo");
        let dep_foo_key = storage.create_key("dep:foo");

        assert_eq!(storage.len(), 1);
        assert_eq!(storage.get(foobar_key), Some("foobar"));
        assert_eq!(storage.get(foo_key), None);
        assert_eq!(storage.get(dep_foo_key), None);
    }
}
