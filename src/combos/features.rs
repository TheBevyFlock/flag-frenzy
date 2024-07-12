use super::Combos;
use crate::intern::{FeatureKey, FeatureStorage};

pub fn feature_combos(
    storage: &FeatureStorage,
    max_k: Option<usize>,
) -> impl Iterator<Item = Box<[FeatureKey]>> {
    let total_features = storage.len();
    let all_keys: Box<[FeatureKey]> = storage.keys().collect();
    let max_k = max_k.unwrap_or(total_features);

    (0..=max_k)
        .flat_map(move |k| Combos::new(total_features, k))
        .map(move |feature_indices| {
            let mut feature_keys = Vec::with_capacity(feature_indices.len());

            for &i in feature_indices.iter() {
                feature_keys.push(all_keys[i]);
            }

            feature_keys.into_boxed_slice()
        })
}
