use super::schema::{self, TrueOrFeatureSet};
use crate::intern::{FeatureKey, FeatureStorage};

/// Represents a feature rule that can be evaluated.
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
    pub fn from_schema(schema: schema::Rule, storage: &mut FeatureStorage) -> Self {
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
/// This is the form of [`FeatureSet`](schema::FeatureSet) that can be evaluated with a
/// combiantion.
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

    /// Parses a [`FeatureSet`](schema::FeatureSet) into a [`FeatureExpr`].
    fn parse(schema: schema::FeatureSet, storage: &mut FeatureStorage) -> Self {
        match schema {
            schema::FeatureSet::One(feature) => {
                // Standalone "OR" operator is not allowed.
                assert_ne!(feature, Self::OR);
                Self::Contains(storage.insert(feature))
            }
            schema::FeatureSet::Many(sets) => {
                if sets.is_empty() {
                    return Self::Always;
                }

                let mut sets = sets.into_iter();
                let mut acc = Self::parse(sets.next().unwrap(), storage);

                while let Some(set) = sets.next() {
                    match set {
                        schema::FeatureSet::One(maybe_or) if maybe_or == Self::OR => {
                            let rhs = sets.next().expect("Expected value after \"OR\".");
                            acc = Self::Or(Box::new(acc), Box::new(Self::parse(rhs, storage)));
                        }
                        schema::FeatureSet::One(feature) => {
                            acc = Self::And(
                                Box::new(acc),
                                Box::new(Self::Contains(storage.insert(feature))),
                            )
                        }
                        schema::FeatureSet::Many(sets) => {
                            acc = Self::And(
                                Box::new(acc),
                                Box::new(Self::parse(schema::FeatureSet::Many(sets), storage)),
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
            Self::Contains(key) => combo.contains(&key),
            Self::And(left, right) => left.evaluate(combo) && right.evaluate(combo),
            Self::Or(left, right) => left.evaluate(combo) || right.evaluate(combo),
        }
    }
}
