use color_eyre::eyre::Context;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use crate::prelude::*;

// Update README if fields change.
#[serde_inline_default]
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde_inline_default(Config::default().disabled_backends)]
    pub disabled_backends: BTreeSet<AnyBackend>,
    #[serde_inline_default(Config::default().arch_package_manager)]
    pub arch_package_manager: ArchPackageManager,
    #[serde_inline_default(Config::default().vscode_variant)]
    pub vscode_variant: VsCodeVariant,
    #[serde_inline_default(Config::default().flatpak_default_systemwide)]
    pub flatpak_default_systemwide: bool,
    #[serde_inline_default(Config::default().hostname_groups_enabled)]
    pub hostname_groups_enabled: bool,
    #[serde_inline_default(Config::default().hostname_groups)]
    pub hostname_groups: BTreeMap<String, Vec<String>>,
}
impl Default for Config {
    fn default() -> Self {
        Config {
            disabled_backends: BTreeSet::new(),
            arch_package_manager: ArchPackageManager::default(),
            vscode_variant: VsCodeVariant::default(),
            flatpak_default_systemwide: true,
            hostname_groups_enabled: false,
            hostname_groups: BTreeMap::new(),
        }
    }
}

impl Config {
    pub fn load(config_dir: &Path) -> Result<Self> {
        let config_file_path = config_dir.join("config.toml");

        if !config_file_path.is_file() {
            log::trace!(
                "no config file found at {config_file_path:?}, using default config instead"
            );

            Ok(Self::default())
        } else {
            toml::from_str(
                &std::fs::read_to_string(config_file_path).wrap_err("reading config file")?,
            )
            .wrap_err("parsing toml config")
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchPackageManager {
    #[default]
    Pacman,
    Pamac,
    Paru,
    Pikaur,
    Yay,
}
impl ArchPackageManager {
    pub fn as_command(&self) -> &'static str {
        match self {
            ArchPackageManager::Pacman => "pacman",
            ArchPackageManager::Pamac => "pamac",
            ArchPackageManager::Paru => "paru",
            ArchPackageManager::Pikaur => "pikaur",
            ArchPackageManager::Yay => "yay",
        }
    }

    pub fn change_perms(&self) -> Perms {
        match self {
            ArchPackageManager::Pacman => Perms::Sudo,
            ArchPackageManager::Pamac => Perms::Same,
            ArchPackageManager::Paru => Perms::Same,
            ArchPackageManager::Pikaur => Perms::Same,
            ArchPackageManager::Yay => Perms::Same,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VsCodeVariant {
    #[default]
    Code,
    Codium,
}
impl VsCodeVariant {
    pub fn as_command(&self) -> &'static str {
        match self {
            VsCodeVariant::Code => "code",
            VsCodeVariant::Codium => "codium",
        }
    }
}
