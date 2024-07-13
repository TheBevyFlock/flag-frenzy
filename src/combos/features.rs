use super::Combos;
use crate::{
    config::RequiredFeature,
    intern::{FeatureKey, FeatureStorage},
};

pub fn feature_combos<'a>(
    storage: &'a FeatureStorage,
    max_k: Option<usize>,
    required: &'a [RequiredFeature],
    incompatible: &'a [Vec<String>],
) -> impl Iterator<Item = Box<[FeatureKey]>> + 'a {
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
        .filter(move |combo| {
            // TODO: Avoid this step by retrieving key from value in storage.
            // Get the string representation of all feature keys, for reference.
            let combo: Vec<_> = combo
                .into_iter()
                .map(|&key| storage.get(key).unwrap())
                .collect();

            for set in required {
                // If the combo does not contain any of the features in the set, skip it.
                if !set
                    .as_slice()
                    .into_iter()
                    .any(|feature| combo.contains(&feature.as_str()))
                {
                    return false;
                }
            }

            for set in incompatible {
                // If all features in an incompatible set are in the combination, skip it.
                if set
                    .into_iter()
                    .all(|feature| combo.contains(&feature.as_str()))
                {
                    return false;
                }
            }

            true
        })
}
