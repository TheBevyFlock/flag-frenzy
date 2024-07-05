use crate::{config::PackageConfigFeatures, intern::{FeatureKey, FeatureStorage}};

use super::Combos;

pub fn feature_combos(storage: &FeatureStorage, config: PackageConfigFeatures) -> impl Iterator<Item = Box<[FeatureKey]>> {
    let total_features = storage.len();

    // The number of features or the max combo size, whichever is smaller.
    let max_k = total_features.min(config.max_combo_size.unwrap_or(usize::MAX));

    let all_keys: Box<[FeatureKey]> = storage.keys().collect();

    (0..=max_k).flat_map(move |k| Combos::new(total_features, k))
        .map(move |feature_indices| {
            let mut feature_keys = Vec::with_capacity(feature_indices.len());

            for &i in feature_indices.into_iter() {
                feature_keys.push(all_keys[i]);
            }

            feature_keys.into_boxed_slice()
        })
}
