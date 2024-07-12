use crate::intern::{FeatureKey, FeatureStorage};
use std::{
    ffi::OsStr,
    path::Path,
    process::{Command, ExitStatus},
};

/// Runs `cargo-check` on a package with the specified features.
pub fn check_with_features(
    package: &str,
    manifest_path: &Path,
    features: &[FeatureKey],
    storage: &FeatureStorage,
) -> ExitStatus {
    // Create comma-separated list of features.
    let features =
        features
            .iter()
            .map(|key| storage.get(*key).unwrap())
            .fold(String::new(), |mut acc, f| {
                acc.push_str(f);
                acc.push(',');
                acc
            });

    Command::new("cargo")
        .arg("check")
        .args([OsStr::new("--manifest-path"), manifest_path.as_os_str()])
        .args(["--package", package])
        .arg("--no-default-features")
        .args(["--features", &features])
        .arg("--quiet")
        .args(["--message-format", "short"])
        .status()
        .unwrap()
}
