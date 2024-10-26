use super::schema::{self, FeatureSet, TrueOrFeatureSet};
use crate::intern::{FeatureKey, FeatureStorage};

/// Represents a feature rule that can be evaluated.
#[derive(PartialEq, Debug)]
pub struct Rule {
    when: FeatureExpr,
    require: FeatureExpr,
    forbid: FeatureExpr,
}

impl Rule {
    /// Creates a new [`Rule`] from the [`schema`](schema::Rule) equivalent.
    ///
    /// This needs a reference to a [`FeatureStorage`] so that it can convert features into
    /// [`FeatureKey`]s.
    pub fn from_schema(schema: schema::Rule, storage: &FeatureStorage) -> Self {
        let when = match schema.when {
            // If true, always run.
            TrueOrFeatureSet::True => FeatureExpr::Always,
            TrueOrFeatureSet::FeatureSet(set) => FeatureExpr::parse(set, storage),
        };

        let require = match schema.require {
            Some(set) => FeatureExpr::parse(set, storage),
            // If there are no requirements, always pass.
            None => FeatureExpr::Always,
        };

        let forbid = match schema.forbid {
            // If true, always fail.
            Some(TrueOrFeatureSet::True) => FeatureExpr::Always,
            // Special case: if the feature set is empty `[]`, never fail. This is because
            // `FeatureExpr` usually evaluates `[]` as `Always`, which we don't want. `forbid = []`
            // should be interpreted as "forbid nothing" instead of "forbid everything".
            Some(TrueOrFeatureSet::FeatureSet(FeatureSet::Many(set))) if set.is_empty() => {
                FeatureExpr::Never
            }
            Some(TrueOrFeatureSet::FeatureSet(set)) => FeatureExpr::parse(set, storage),
            // If there are no forbidden, never fail.
            None => FeatureExpr::Never,
        };

        Self {
            when,
            require,
            forbid,
        }
    }

    /// Returns true if the features in a given combination passes this rule.
    pub fn validate(&self, combo: &[FeatureKey]) -> bool {
        if self.when.evaluate(combo) {
            self.require.evaluate(combo) && !self.forbid.evaluate(combo)
        } else {
            true
        }
    }
}

/// A recursive expression of feature requirements.
///
/// This is the form of [`FeatureSet`] that can be evaluated with a
/// combiantion.
#[derive(PartialEq, Debug)]
enum FeatureExpr {
    /// Always evaluates as true.
    Always,
    /// Always evaluates as false.
    Never,
    /// Only evaluates as true if the given combination contains the [`FeatureKey`].
    Contains(FeatureKey),
    /// Only evaluates as true if both expressions are true.
    And(Box<FeatureExpr>, Box<FeatureExpr>),
    /// Only evaluates as true if at least one expression is true.
    Or(Box<FeatureExpr>, Box<FeatureExpr>),
}

impl FeatureExpr {
    /// The constant "OR", used as a separate to determine if two feature should be or'd together.
    const OR: &'static str = "OR";

    /// Parses a [`FeatureSet`] into a [`FeatureExpr`].
    fn parse(schema: FeatureSet, storage: &FeatureStorage) -> Self {
        match schema {
            FeatureSet::One(feature) => {
                // Standalone "OR" operator is not allowed.
                assert_ne!(feature, Self::OR, "Feature set cannot begin with \"OR\"");
                Self::Contains(storage.create_key(&feature))
            }
            FeatureSet::Many(sets) => {
                // Empty sets always pass. (Note that `Rule::from_schema()` special cases the
                // `forbid` property to make this `Never`.)
                if sets.is_empty() {
                    return Self::Always;
                }

                let mut sets = sets.into_iter();
                let mut acc = Self::parse(sets.next().unwrap(), storage);

                while let Some(set) = sets.next() {
                    match set {
                        FeatureSet::One(maybe_or) if maybe_or == Self::OR => {
                            let rhs = sets.next().expect("Expected value after \"OR\".");
                            acc = Self::Or(Box::new(acc), Box::new(Self::parse(rhs, storage)));
                        }
                        FeatureSet::One(feature) => {
                            acc = Self::And(
                                Box::new(acc),
                                Box::new(Self::Contains(storage.create_key(&feature))),
                            )
                        }
                        FeatureSet::Many(sets) => {
                            acc = Self::And(
                                Box::new(acc),
                                Box::new(Self::parse(FeatureSet::Many(sets), storage)),
                            )
                        }
                    }
                }

                acc
            }
        }
    }

    /// Evaluates this expression for a given combination.
    fn evaluate(&self, combo: &[FeatureKey]) -> bool {
        match self {
            Self::Always => true,
            Self::Never => false,
            Self::Contains(key) => combo.contains(key),
            Self::And(left, right) => left.evaluate(combo) && right.evaluate(combo),
            Self::Or(left, right) => left.evaluate(combo) || right.evaluate(combo),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use serde_json::json;

    #[test]
    fn rule_from_schema() {
        let mut storage = FeatureStorage::new();

        let foo = storage.insert("foo".to_string(), &HashMap::new());
        let bar = storage.insert("bar".to_string(), &HashMap::new());
        let baz = storage.insert("baz".to_string(), &HashMap::new());

        let empty = Rule::from_schema(
            schema::Rule {
                when: TrueOrFeatureSet::True,
                require: None,
                forbid: None,
            },
            &mut storage,
        );

        assert_eq!(
            empty,
            Rule {
                when: FeatureExpr::Always,
                require: FeatureExpr::Always,
                forbid: FeatureExpr::Never,
            }
        );

        let always_forbid = Rule::from_schema(
            schema::Rule {
                when: TrueOrFeatureSet::True,
                require: None,
                forbid: Some(TrueOrFeatureSet::True),
            },
            &mut storage,
        );

        assert_eq!(
            always_forbid,
            Rule {
                when: FeatureExpr::Always,
                require: FeatureExpr::Always,
                forbid: FeatureExpr::Always,
            }
        );

        let empty_forbid = Rule::from_schema(
            schema::Rule {
                when: TrueOrFeatureSet::True,
                require: None,
                forbid: Some(TrueOrFeatureSet::FeatureSet(FeatureSet::Many(Vec::new()))),
            },
            &mut storage,
        );

        assert_eq!(
            empty_forbid,
            Rule {
                when: FeatureExpr::Always,
                require: FeatureExpr::Always,
                forbid: FeatureExpr::Never,
            }
        );

        let normal = Rule::from_schema(
            schema::Rule {
                when: TrueOrFeatureSet::FeatureSet(FeatureSet::One("foo".to_string())),
                require: Some(FeatureSet::One("bar".to_string())),
                forbid: Some(TrueOrFeatureSet::FeatureSet(FeatureSet::One(
                    "baz".to_string(),
                ))),
            },
            &mut storage,
        );

        assert_eq!(
            normal,
            Rule {
                when: FeatureExpr::Contains(foo),
                require: FeatureExpr::Contains(bar),
                forbid: FeatureExpr::Contains(baz),
            }
        );
    }

    #[test]
    fn rule_validate() {
        let mut storage = FeatureStorage::new();

        let foo = storage.insert("foo".to_string(), &HashMap::new());
        let bar = storage.insert("bar".to_string(), &HashMap::new());
        let baz = storage.insert("baz".to_string(), &HashMap::new());

        let always_deny_foo = Rule {
            when: FeatureExpr::Always,
            require: FeatureExpr::Always,
            forbid: FeatureExpr::Contains(foo),
        };

        assert!(!always_deny_foo.validate(&[foo]));
        assert!(always_deny_foo.validate(&[]));
        assert!(always_deny_foo.validate(&[bar, baz]));

        let require_bar_or_baz_when_foo = Rule {
            when: FeatureExpr::Contains(foo),
            require: FeatureExpr::Or(
                Box::new(FeatureExpr::Contains(bar)),
                Box::new(FeatureExpr::Contains(baz)),
            ),
            forbid: FeatureExpr::Never,
        };

        assert!(require_bar_or_baz_when_foo.validate(&[]));
        assert!(!require_bar_or_baz_when_foo.validate(&[foo]));
        assert!(require_bar_or_baz_when_foo.validate(&[foo, bar]));
        assert!(require_bar_or_baz_when_foo.validate(&[foo, bar, baz]));

        let bar_baz_incompatible = Rule {
            when: FeatureExpr::And(
                Box::new(FeatureExpr::Contains(bar)),
                Box::new(FeatureExpr::Contains(baz)),
            ),
            require: FeatureExpr::Always,
            forbid: FeatureExpr::Always,
        };

        assert!(bar_baz_incompatible.validate(&[]));
        assert!(bar_baz_incompatible.validate(&[bar]));
        assert!(bar_baz_incompatible.validate(&[baz]));
        assert!(!bar_baz_incompatible.validate(&[bar, baz]));
    }

    #[test]
    fn parse_expression() {
        fn expr_from_json(value: serde_json::Value, storage: &mut FeatureStorage) -> FeatureExpr {
            FeatureExpr::parse(serde_json::from_value(value).unwrap(), storage)
        }

        let mut storage = FeatureStorage::new();

        let foo = storage.insert("foo".to_string(), &HashMap::new());
        let bar = storage.insert("bar".to_string(), &HashMap::new());
        let baz = storage.insert("baz".to_string(), &HashMap::new());

        let empty = expr_from_json(json!([]), &mut storage);

        assert_eq!(empty, FeatureExpr::Always);

        let one = expr_from_json(json!("foo"), &mut storage);

        assert_eq!(one, FeatureExpr::Contains(foo));

        let and_2 = expr_from_json(json!(["foo", "bar"]), &mut storage);

        assert_eq!(
            and_2,
            FeatureExpr::And(
                Box::new(FeatureExpr::Contains(foo)),
                Box::new(FeatureExpr::Contains(bar)),
            ),
        );

        let and_3 = expr_from_json(json!(["foo", "bar", "baz"]), &mut storage);

        assert_eq!(
            and_3,
            FeatureExpr::And(
                Box::new(FeatureExpr::And(
                    Box::new(FeatureExpr::Contains(foo)),
                    Box::new(FeatureExpr::Contains(bar)),
                )),
                Box::new(FeatureExpr::Contains(baz)),
            ),
        );

        let or_2 = expr_from_json(json!(["foo", "OR", "bar"]), &mut storage);

        assert_eq!(
            or_2,
            FeatureExpr::Or(
                Box::new(FeatureExpr::Contains(foo)),
                Box::new(FeatureExpr::Contains(bar)),
            ),
        );

        let or_3 = expr_from_json(json!(["foo", "OR", "bar", "OR", "baz"]), &mut storage);

        assert_eq!(
            or_3,
            FeatureExpr::Or(
                Box::new(FeatureExpr::Or(
                    Box::new(FeatureExpr::Contains(foo)),
                    Box::new(FeatureExpr::Contains(bar)),
                )),
                Box::new(FeatureExpr::Contains(baz)),
            ),
        );

        // Evaluated left-to-right, equivalent to `(foo || bar) && baz`.
        let and_or_mixed = expr_from_json(json!(["foo", "OR", "bar", "baz"]), &mut storage);

        assert_eq!(
            and_or_mixed,
            FeatureExpr::And(
                Box::new(FeatureExpr::Or(
                    Box::new(FeatureExpr::Contains(foo)),
                    Box::new(FeatureExpr::Contains(bar)),
                )),
                Box::new(FeatureExpr::Contains(baz)),
            ),
        );

        // The square brackets get evaluated first, equivalent to `foo || (bar && baz)`.
        let explicit_oop = expr_from_json(json!(["foo", "OR", ["bar", "baz"]]), &mut storage);

        assert_eq!(
            explicit_oop,
            FeatureExpr::Or(
                Box::new(FeatureExpr::Contains(foo)),
                Box::new(FeatureExpr::And(
                    Box::new(FeatureExpr::Contains(bar)),
                    Box::new(FeatureExpr::Contains(baz)),
                )),
            ),
        );

        let depth_unwinding = expr_from_json(
            json!([[[[[["foo"]]]], [[[[[[[["bar"]]]]]]]]]]),
            &mut storage,
        );

        assert_eq!(
            depth_unwinding,
            FeatureExpr::And(
                Box::new(FeatureExpr::Contains(foo)),
                Box::new(FeatureExpr::Contains(bar)),
            ),
        );
    }

    #[test]
    fn evaluate_expression() {
        let mut storage = FeatureStorage::new();

        let foo = storage.insert("foo".to_string(), &HashMap::new());
        let bar = storage.insert("bar".to_string(), &HashMap::new());
        let baz = storage.insert("baz".to_string(), &HashMap::new());

        let always = FeatureExpr::Always;

        assert!(always.evaluate(&[]));
        assert!(always.evaluate(&[foo]));

        let never = FeatureExpr::Never;

        assert!(!never.evaluate(&[]));
        assert!(!never.evaluate(&[foo]));

        let contains = FeatureExpr::Contains(foo);

        assert!(contains.evaluate(&[foo]));
        assert!(contains.evaluate(&[foo, bar]));
        assert!(!contains.evaluate(&[]));
        assert!(!contains.evaluate(&[bar]));

        let and = FeatureExpr::And(
            Box::new(FeatureExpr::Contains(foo)),
            Box::new(FeatureExpr::Contains(bar)),
        );

        assert!(and.evaluate(&[foo, bar]));
        assert!(and.evaluate(&[foo, bar, baz]));
        assert!(!and.evaluate(&[foo]));
        assert!(!and.evaluate(&[]));

        let or = FeatureExpr::Or(
            Box::new(FeatureExpr::Contains(foo)),
            Box::new(FeatureExpr::Contains(bar)),
        );

        assert!(or.evaluate(&[foo]));
        assert!(or.evaluate(&[bar]));
        assert!(or.evaluate(&[foo, bar]));
        assert!(!or.evaluate(&[]));
    }
}
