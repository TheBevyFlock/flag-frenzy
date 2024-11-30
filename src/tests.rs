use std::path::{Path, PathBuf};

use crate::{
    cli::CLI,
    combos::feature_combos,
    config::{load_config, WorkspaceConfig},
    intern::intern_features,
    manifest::{load_manifest, Package},
    process_packages,
};

fn test(manifest_path: &Path, expected_combos: Vec<Vec<&str>>, config_path: Option<&Path>) {
    let config = config_path
        .map(|path| load_config(path).unwrap())
        .unwrap_or_else(|| WorkspaceConfig::default());
    let path = PathBuf::from(manifest_path);
    let cli = CLI {
        manifest_path: path.clone(),
        config: None,
        package: None,
        chunk: None,
        total_chunks: None,
        color: crate::cli::ColorChoice::Always,
        dry_run: false,
    };
    let manifest = load_manifest(&path).unwrap();
    let mut packages = process_packages(manifest, &cli, &config).unwrap();
    let Package { name, features } = packages.pop().unwrap();
    let package_config = config.get(&name);
    let storage = intern_features(features, package_config);
    let mut combos: Vec<_> = feature_combos(&storage, package_config)
        .map(|feature_keys| {
            let mut features: Vec<_> = feature_keys
                .iter()
                .map(|feature_key| storage.get(*feature_key).unwrap())
                .collect();
            features.sort_unstable();
            features
        })
        .collect();
    combos.sort_unstable();
    assert_eq!(combos, expected_combos);
}

#[test]
fn parse_empty() {
    test(Path::new("tests/empty/Cargo.toml"), vec![vec![]], None);
}

#[test]
fn parse_simple() {
    test(
        Path::new("tests/simple/Cargo.toml"),
        vec![
            vec![],
            vec!["bar"],
            vec!["bar", "foo"],
            vec!["baz"],
            vec!["baz", "foo"],
            vec!["foo"],
        ],
        None,
    );
}

#[test]
fn parse_complex() {
    test(
        Path::new("tests/complex/Cargo.toml"),
        vec![
            vec!["always-required", "choose-required-1"],
            vec!["always-required", "choose-required-1", "choose-required-2"],
            vec![
                "always-required",
                "choose-required-1",
                "choose-required-2",
                "incompatible-1",
            ],
            vec![
                "always-required",
                "choose-required-1",
                "choose-required-2",
                "incompatible-2",
            ],
            vec!["always-required", "choose-required-1", "incompatible-1"],
            vec!["always-required", "choose-required-1", "incompatible-2"],
            vec!["always-required", "choose-required-2"],
            vec!["always-required", "choose-required-2", "incompatible-1"],
            vec!["always-required", "choose-required-2", "incompatible-2"],
        ],
        Some(Path::new("tests/complex/config")),
    );
}

#[test]
fn parse_dependencies() {
    test(
        Path::new("tests/dependencies/Cargo.toml"),
        vec![
            vec![],
            vec!["contains-dependencies"],
            vec!["contains-dependencies", "simple"],
            vec!["dependency1"],
            vec!["dependency1", "dependency2"],
            vec!["dependency1", "dependency2", "simple"],
            vec!["dependency1", "empty"],
            vec!["dependency1", "empty", "simple"],
            vec!["dependency1", "simple"],
            vec!["dependency2"],
            vec!["dependency2", "dependency3"],
            vec!["dependency2", "dependency3", "simple"],
            vec!["dependency2", "simple"],
            vec!["dependency3"],
            vec!["dependency3", "empty"],
            vec!["dependency3", "empty", "simple"],
            vec!["dependency3", "simple"],
            vec!["empty"],
            vec!["empty", "simple"],
            vec!["simple"],
        ],
        None,
    );
}

#[test]
fn parse_optional_dependencies_without_filtering_used_dependencies() {
    test(
        Path::new("tests/optional-dependencies/Cargo.toml"),
        vec![
            vec![],
            vec!["explicit"],
            vec!["explicit", "foo"],
            vec!["explicit", "foo", "implicit"],
            vec!["explicit", "implicit"],
            vec!["explicit", "implicit", "used"],
            vec!["explicit", "used"],
            vec!["foo"],
            vec!["foo", "implicit"],
            vec!["implicit"],
            vec!["implicit", "used"],
            vec!["used"],
        ],
        None,
    );
}

#[test]
fn parse_optional_dependencies_and_filter_used_dependencies() {
    test(
        Path::new("tests/optional-dependencies/Cargo.toml"),
        vec![
            vec![],
            vec!["explicit"],
            vec!["explicit", "foo"],
            vec!["explicit", "foo", "implicit"],
            vec!["explicit", "implicit"],
            vec!["foo"],
            vec!["foo", "implicit"],
            vec!["implicit"],
        ],
        Some(Path::new("tests/optional-dependencies/config")),
    );
}
