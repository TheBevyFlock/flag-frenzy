use std::{
    ffi::OsStr,
    path::Path,
    process::{Command, Output},
};

use crate::intern::{FeatureKey, FeatureStorage};

/// Runs `cargo-check` on a package with the specified features.
pub fn check_with_features(
    package: &str,
    manifest_path: &Path,
    features: &[FeatureKey],
    storage: &FeatureStorage,
) -> Output {
    // Create comma-separated list of features.
    let features = features
        .into_iter()
        .map(|key| storage.get(*key).unwrap())
        .fold(String::new(), |mut acc, f| {
            acc.push_str(f);
            acc.push(',');
            acc
        });

    // Strip ending comma if it exists.
    let features = match features.strip_suffix(',') {
        Some(f) => f,
        None => &features,
    };

    println!("Checking {package} with features {features}");

    Command::new("cargo")
        .arg("check")
        .args([OsStr::new("--manifest-path"), manifest_path.as_os_str()])
        .args(["--package", package])
        .args(["--color", "never"])
        .arg("--no-default-features")
        .args(["--features", features])
        .output()
        .unwrap()
}
