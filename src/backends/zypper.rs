use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Zypper;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ZypperConfig {
    #[serde(default)]
    pub distribution_upgrade: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZypperPackageOptions {}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZypperRepoOptions {}

impl Backend for Zypper {
    type Config = ZypperConfig;
    type PackageOptions = ZypperPackageOptions;
    type RepoOptions = ZypperRepoOptions;

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
    ) -> Result<std::collections::BTreeMap<String, Self::PackageOptions>> {
        if Self::version(config).is_err() {
            return Ok(BTreeMap::new());
        }

        let stdout = run_command_for_stdout(
            ["zypper", "packages", "--userinstalled"],
            Perms::Same,
            StdErr::Show,
        )?;

        stdout
            .lines()
            .filter(|line| line.starts_with("i+"))
            .map(|line| -> Result<(String, Self::PackageOptions)> {
                let mut parts = line.split('|');
                let package = parts
                    .nth(2)
                    .ok_or(eyre!("unexpected output"))?
                    .trim()
                    .to_string();
                Ok((package, Self::PackageOptions {}))
            })
            .collect()
    }

    fn install_packages(
        packages: &BTreeMap<String, Self::PackageOptions>,
        no_confirm: bool,
        _: &Self::Config,
    ) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["zypper", "install"]
                    .into_iter()
                    .chain(Some("--no-confirm").filter(|_| no_confirm))
                    .chain(packages.keys().map(String::as_str)),
                Perms::Sudo,
            )?;
        }

        Ok(())
    }

    fn uninstall_packages(
        packages: &BTreeSet<String>,
        no_confirm: bool,
        _: &Self::Config,
    ) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["zypper", "remove"]
                    .into_iter()
                    .chain(Some("--no-confirm").filter(|_| no_confirm))
                    .chain(packages.iter().map(String::as_str)),
                Perms::Sudo,
            )?;
        }

        Ok(())
    }

    fn update_packages(
        packages: &BTreeSet<String>,
        no_confirm: bool,
        _: &Self::Config,
    ) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["zypper", "update"]
                    .into_iter()
                    .chain(Some("--no-confirm").filter(|_| no_confirm))
                    .chain(packages.iter().map(String::as_str)),
                Perms::Sudo,
            )?;
        }

        Ok(())
    }

    fn update_all_packages(no_confirm: bool, config: &Self::Config) -> Result<()> {
        run_command(
            [
                "zypper",
                if !config.distribution_upgrade {
                    "update"
                } else {
                    "dist-upgrade"
                },
            ]
            .into_iter()
            .chain(Some("--no-confirm").filter(|_| no_confirm)),
            Perms::Sudo,
        )
    }

    fn clean_cache(config: &Self::Config) -> Result<()> {
        Self::version(config).map_or(Ok(()), |_| run_command(["zypper", "clean"], Perms::Sudo))
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

    fn version(_: &Self::Config) -> Result<String> {
        run_command_for_stdout(["zypper", "--version"], Perms::Same, StdErr::Show)
    }
}
