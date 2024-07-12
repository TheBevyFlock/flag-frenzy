#[allow(clippy::module_inception)]
mod combos;
mod features;
mod ncr;

pub use self::combos::Combos;
pub use self::features::feature_combos;
pub use self::ncr::{estimate_combos, ncr};
