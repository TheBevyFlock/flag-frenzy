#[allow(clippy::module_inception)]
mod combos;
mod ncr;
mod features;

pub use self::combos::Combos;
pub use self::ncr::ncr;
pub use self::features::feature_combos;
