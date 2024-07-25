use super::Combos;
use crate::{
    config::{Config, Rule},
    intern::{FeatureKey, FeatureStorage},
};

pub fn feature_combos<'a>(
    storage: &'a mut FeatureStorage,
    config: Config<'_>,
) -> impl Iterator<Item = Box<[FeatureKey]>> + 'a {
    let total_features = storage.len();
    let all_keys: Box<[_]> = storage.keys().collect();

    let max_k = config
        .max_combo_size()
        .unwrap_or(total_features)
        .min(total_features);

    let rules: Box<[_]> = config
        .rules()
        .iter()
        .cloned() // TODO: Do not clone this.
        .map(|r| Rule::from_schema(r, storage))
        .collect();

    (0..=max_k)
        // Flatten all combinations of `(n: total_features, k: 0..=max_k)`.
        .flat_map(move |k| Combos::new(total_features, k))
        // Convert arrays of `usize` indices to actual `FeatureKey`s.
        .map(move |feature_indices| {
            let mut feature_keys = Vec::with_capacity(feature_indices.len());

            for &i in feature_indices.iter() {
                feature_keys.push(all_keys[i]);
            }

            feature_keys.into_boxed_slice()
        })
        // Only yield combinations that pass all rules for this crate.
        .filter(move |combo| rules.iter().all(|r| r.validate(combo)))
}
