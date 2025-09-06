use crate::prelude::*;
use color_eyre::Result;
use color_eyre::eyre::{Context, eyre};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Config {
    // update README if fields change.
    #[serde(default)]
    pub enabled_backends: BTreeSet<AnyBackend>,
    #[serde(default)]
    pub hostname_groups_enabled: bool,
    #[serde(default)]
    pub hostname_groups: BTreeMap<String, Vec<String>>,
    #[serde(flatten)]
    pub backends: BackendConfigs,
}
impl Config {
    pub fn load(config_dir: &Path) -> Result<Self> {
        let config_file_path = config_dir.join("config.toml");

        if !config_file_path.is_file() {
            log::warn!(
                "no config file found at {config_file_path:?}, using default config instead"
            );

            Ok(Self::default())
        } else {
            toml::from_str(
                &std::fs::read_to_string(config_file_path.clone())
                    .wrap_err("reading config file")?,
            )
            .wrap_err(eyre!("parsing toml config {config_file_path:?}"))
        }
    }
}
