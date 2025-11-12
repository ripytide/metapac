use crate::prelude::*;
use color_eyre::Result;
use color_eyre::eyre::{Context, ContextCompat, eyre};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Config {
    // update README.md if fields change.
    #[serde(default)]
    enabled_backends: BTreeSet<AnyBackend>,
    #[serde(default)]
    hostname_groups_enabled: bool,
    #[serde(default)]
    hostname_enabled_backends: BTreeMap<String, BTreeSet<AnyBackend>>,
    #[serde(default)]
    hostname_groups: BTreeMap<String, Vec<String>>,
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

    pub fn enabled_backends(&self, hostname: &str) -> BTreeSet<AnyBackend> {
        let mut backends = self.enabled_backends.clone();
        if let Some(host_backends) = self.hostname_enabled_backends.get(hostname) {
            for host_backend in host_backends {
                if !backends.contains(host_backend) {
                    backends.insert(*host_backend);
                }
            }
        }
        backends
    }

    pub fn group_files(&self, group_dir: &Path, hostname: &str) -> Result<BTreeSet<PathBuf>> {
        if self.hostname_groups_enabled {
            let group_names = self.hostname_groups.get(hostname).wrap_err(eyre!(
                "no entry in the `hostname_groups` config for the hostname: {hostname:?}"
            ))?;

            Ok(group_names
                .iter()
                .map(|group_name| group_dir.join(group_name).with_extension("toml"))
                .collect())
        } else {
            if !group_dir.is_dir() {
                log::warn!(
                    "the groups directory: {group_dir:?}, was not found, assuming there are no group files. If this was intentional please create an empty groups folder."
                );

                return Ok(BTreeSet::new());
            }

            Ok(walkdir::WalkDir::new(group_dir)
                .follow_links(true)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|x| !x.file_type().is_dir())
                .map(|x| x.path().to_path_buf())
                .collect())
        }
    }
}
