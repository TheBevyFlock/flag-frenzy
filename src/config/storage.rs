use super::schema;
use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct WorkspaceConfig {
    crates: HashMap<String, CrateConfig>,

    max_combo_size: Option<usize>,
    skip_optional_deps: Option<bool>,
}

impl WorkspaceConfig {
    pub fn new(crates: HashMap<String, CrateConfig>, global: schema::Config) -> Self {
        let schema::Config {
            max_combo_size,
            skip_optional_deps,
            rules: _,
        } = global;

        Self {
            crates,
            max_combo_size,
            skip_optional_deps,
        }
    }

    pub fn get(&self, name: &str) -> Config<'_> {
        Config {
            workspace: self,
            crate_: self.crates.get(name),
        }
    }
}

#[derive(Debug)]
pub struct CrateConfig {
    max_combo_size: Option<usize>,
    skip_optional_deps: Option<bool>,
    rules: Vec<schema::Rule>,
}

impl From<schema::Config> for CrateConfig {
    fn from(value: schema::Config) -> Self {
        let schema::Config {
            max_combo_size,
            skip_optional_deps,
            rules,
        } = value;

        Self {
            max_combo_size,
            skip_optional_deps,
            rules,
        }
    }
}

#[derive(Debug)]
pub struct Config<'a> {
    workspace: &'a WorkspaceConfig,
    crate_: Option<&'a CrateConfig>,
}

impl<'a> Config<'a> {
    /// Defaults to [`None`].
    pub fn max_combo_size(&self) -> Option<usize> {
        self.crate_
            .map_or(None, |c| c.max_combo_size)
            .or(self.workspace.max_combo_size)
    }

    /// Defaults to false.
    pub fn skip_optional_deps(&self) -> bool {
        self.crate_
            .map_or(None, |c| c.skip_optional_deps)
            .or(self.workspace.skip_optional_deps)
            .unwrap_or_default()
    }

    /// Defaults to an empty slice.
    pub fn rules(&self) -> &[schema::Rule] {
        self.crate_.map_or(&[], |c| &c.rules)
    }
}
