use slotmap::{new_key_type, SlotMap};

new_key_type! { pub struct FeatureKey; }

pub type Features = SlotMap<FeatureKey, String>;
