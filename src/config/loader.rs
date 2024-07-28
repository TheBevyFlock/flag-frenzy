use super::storage::WorkspaceConfig;
use crate::config::{schema, storage::CrateConfig};
use anyhow::{ensure, Context};
use std::{collections::HashMap, fs, path::Path};

/// Loads all crate configuration within a given folder.
///
/// This will only load files (not symlinks) with a `.toml` extension, all other will be skipped.
/// The file `global.toml` is special-cased: it cannot contain any rules, but it will provide the
/// new defaults for all other crate configuration.
pub fn load_config(folder: &Path) -> anyhow::Result<WorkspaceConfig> {
    let mut global = schema::Config::default();
    let mut crates = HashMap::new();

    for file in fs::read_dir(folder)? {
        let file = file?;
        let mut name = file.file_name().to_string_lossy().into_owned();

        // Filter to only TOML files. Note that symlinks are currently skipped. If the need arises,
        // this can be implemented.
        if !(file.file_type()?.is_file() && name.ends_with(".toml")) {
            continue;
        }

        let path = file.path();

        // Remove ".toml" from the end of the name.
        name.truncate(name.len() - 5);

        ensure!(!name.is_empty(), "Config file {path:?} cannot be named \".toml\" because the name determines the affected package.");

        let contents = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {path:?} to a string."))?;

        let config: schema::Config = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse {path:?} as TOML."))?;

        if name == "global" {
            ensure!(
                config.rules.is_empty(),
                "Config \"global.toml\" cannot define rules, as they will not be inherited."
            );

            global = config;
            continue;
        }

        crates.insert(name, CrateConfig::from(config));
    }

    Ok(WorkspaceConfig::new(crates, global))
}
