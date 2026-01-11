use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::cmd::run_command;
use crate::cmd::run_command_for_stdout;
use crate::prelude::*;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct VsCode;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct VsCodeConfig {
    #[serde(default)]
    pub variant: VsCodeVariant,
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

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VsCodePackageOptions {}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VsCodeRepoOptions {}

impl Backend for VsCode {
    type Config = VsCodeConfig;
    type PackageOptions = VsCodePackageOptions;
    type RepoOptions = VsCodeRepoOptions;

    fn invalid_package_help_text() -> String {
        String::new()
    }

    fn is_valid_package_name(_: &str) -> Option<bool> {
        None
    }

    fn get_all_packages(_: &Self::Config) -> Result<BTreeSet<String>> {
        Err(eyre!("unimplemented"))
    }

    fn get_installed_packages(
        config: &Self::Config,
    ) -> Result<BTreeMap<String, Self::PackageOptions>> {
        if Self::version(config).is_err() {
            return Ok(BTreeMap::new());
        }

        let names = run_command_for_stdout(
            [config.variant.as_command(), "--list-extensions"],
            Perms::Same,
            true,
        )?
        .lines()
        .map(|x| (x.to_string(), Self::PackageOptions {}))
        .collect();

        Ok(names)
    }

    fn install_packages(
        packages: &BTreeMap<String, Self::PackageOptions>,
        _: bool,
        config: &Self::Config,
    ) -> Result<()> {
        for package in packages.keys() {
            run_command(
                [config.variant.as_command(), "--install-extension", package],
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn uninstall_packages(
        packages: &BTreeSet<String>,
        _: bool,
        config: &Self::Config,
    ) -> Result<()> {
        for package in packages {
            run_command(
                [
                    config.variant.as_command(),
                    "--uninstall-extension",
                    package,
                ],
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn update_packages(packages: &BTreeSet<String>, _: bool, config: &Self::Config) -> Result<()> {
        for package in packages {
            run_command(
                [config.variant.as_command(), "--install-extension", package],
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn update_all_packages(no_confirm: bool, config: &Self::Config) -> Result<()> {
        let packages = Self::get_installed_packages(config)?;
        Self::update_packages(
            &packages.keys().map(String::from).collect(),
            no_confirm,
            config,
        )
    }

    fn clean_cache(_: &Self::Config) -> Result<()> {
        Ok(())
    }

    fn get_installed_repos(_: &Self::Config) -> Result<BTreeMap<String, Self::RepoOptions>> {
        Ok(BTreeMap::new())
    }

    fn add_repos(_: &BTreeMap<String, Self::RepoOptions>, _: bool, _: &Self::Config) -> Result<()> {
        Err(eyre!("unimplemented"))
    }

    fn remove_repos(_: &BTreeSet<String>, _: bool, _: &Self::Config) -> Result<()> {
        Err(eyre!("unimplemented"))
    }

    fn version(config: &Self::Config) -> Result<String> {
        run_command_for_stdout(
            [config.variant.as_command(), "--version"],
            Perms::Same,
            false,
        )
        .map(|x| x.lines().join(" "))
    }
}
