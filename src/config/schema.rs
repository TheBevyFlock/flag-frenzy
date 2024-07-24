use serde::{de::Error, Deserialize, Deserializer};

/// Represents the configuration for a specific crate.
#[derive(Deserialize, Default, Debug)]
pub struct Config {
    pub max_combo_size: Option<usize>,
    pub skip_optional_deps: Option<bool>,

    #[serde(default, rename = "rule")]
    pub rules: Vec<Rule>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct Rule {
    pub when: TrueOrFeatureSet,
    pub require: Option<FeatureSet>,
    pub forbid: Option<TrueOrFeatureSet>,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(untagged)]
pub enum TrueOrFeatureSet {
    #[serde(deserialize_with = "deserialize_true")]
    True,
    FeatureSet(FeatureSet),
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(untagged)]
pub enum FeatureSet {
    One(String),
    Many(Vec<FeatureSet>),
}

fn deserialize_true<'de, D>(d: D) -> Result<(), D::Error>
where
    D: Deserializer<'de>,
{
    match bool::deserialize(d) {
        // Is true, we're good to go!
        Ok(true) => Ok(()),
        // Is false, let's pretend it's invalid.
        Ok(false) => Err(D::Error::invalid_type(
            serde::de::Unexpected::Bool(false),
            &"the boolean `true`, a string, or an array of strings (feature set)",
        )),
        // Is not a boolean, return the error.
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn rule() {
        let always_nothing: Rule = serde_json::from_value(json!({ "when": true })).unwrap();
        assert_eq!(
            always_nothing,
            Rule {
                when: TrueOrFeatureSet::True,
                require: None,
                forbid: None,
            },
        );
    }

    #[test]
    fn true_or_feature_set() {
        let true_: TrueOrFeatureSet = serde_json::from_value(json!(true)).unwrap();
        assert_eq!(true_, TrueOrFeatureSet::True);

        let false_: Result<TrueOrFeatureSet, _> = serde_json::from_value(json!(false));
        assert!(false_.is_err());

        let feature_set_one: TrueOrFeatureSet = serde_json::from_value(json!("foo")).unwrap();
        assert_eq!(
            feature_set_one,
            TrueOrFeatureSet::FeatureSet(FeatureSet::One("foo".to_string())),
        );

        let feature_set_many: TrueOrFeatureSet =
            serde_json::from_value(json!(["foo", "bar"])).unwrap();
        assert_eq!(
            feature_set_many,
            TrueOrFeatureSet::FeatureSet(FeatureSet::Many(vec![
                FeatureSet::One("foo".to_string()),
                FeatureSet::One("bar".to_string()),
            ])),
        );
    }

    #[test]
    fn feature_set() {
        let one: FeatureSet = serde_json::from_value(json!("foo")).unwrap();
        assert_eq!(one, FeatureSet::One("foo".to_string()));

        let many: FeatureSet = serde_json::from_value(json!(["foo", "OR", "bar"])).unwrap();
        assert_eq!(
            many,
            FeatureSet::Many(vec![
                FeatureSet::One("foo".to_string()),
                FeatureSet::One("OR".to_string()),
                FeatureSet::One("bar".to_string()),
            ]),
        );

        let many_nested: FeatureSet =
            serde_json::from_value(json!([["foo", "OR", ["bar"]], "baz"])).unwrap();
        assert_eq!(
            many_nested,
            FeatureSet::Many(vec![
                FeatureSet::Many(vec![
                    FeatureSet::One("foo".to_string()),
                    FeatureSet::One("OR".to_string()),
                    FeatureSet::Many(vec![FeatureSet::One("bar".to_string())]),
                ]),
                FeatureSet::One("baz".to_string()),
            ]),
        );
    }
}
