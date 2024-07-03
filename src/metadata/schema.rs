use std::{collections::HashMap, path::PathBuf};

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Metadata {
    pub packages: Vec<Package>,
}

#[derive(Deserialize, Debug)]
pub struct Package {
    pub name: String,
    pub features: HashMap<String, Vec<String>>,
    pub manifest_path: PathBuf,
}
