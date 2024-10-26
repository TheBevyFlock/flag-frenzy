use super::Combos;
use crate::{
    config::{Config, Rule},
    intern::{FeatureKey, FeatureStorage},
};

pub fn feature_combos<'a>(
    storage: &'a FeatureStorage,
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
        // Only yield combinations that don't contain dependencies.
        .filter(move |combo| {
            combo.iter().all(|key| {
                combo
                    .iter()
                    .all(|other_key| !storage.is_dependency(*key, *other_key))
            })
        })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{config::WorkspaceConfig, intern::intern_features};

    use super::feature_combos;

    #[test]
    fn test_filter_dependencies() {
        let mut features_map = HashMap::new();
        features_map.insert("foo".to_string(), Vec::new());
        features_map.insert("bar".to_string(), vec!["foo".to_string()]);
        features_map.insert("foobar".to_string(), vec!["bar".to_string()]);
        features_map.insert("unrelated".to_string(), Vec::new());

        let schema_config = crate::config::schema::Config {
            max_combo_size: Some(3),
            skip_optional_deps: None,
            rules: vec![],
        };
        let workspace_config = WorkspaceConfig::new(HashMap::new(), schema_config);
        let storage = intern_features(features_map, workspace_config.get(""));

        let mut combos: Vec<_> = feature_combos(&storage, workspace_config.get("foo"))
            .map(|combo| {
                let mut vec = combo
                    .iter()
                    .map(|key| storage.get(*key).unwrap())
                    .collect::<Vec<_>>();
                vec.sort();
                vec
            })
            .collect();
        combos.sort();

        let mut expected = vec![
            vec![],
            vec!["foo"],
            vec!["bar"],
            vec!["unrelated"],
            vec!["foobar"],
            vec!["unrelated", "foo"],
            vec!["unrelated", "bar"],
            vec!["unrelated", "foobar"],
        ];
        for combo in &mut expected {
            combo.sort();
        }
        expected.sort();

        assert_eq!(combos, expected);
    }
}
