use std::collections::{BTreeMap, BTreeSet};

use color_eyre::Result;
use color_eyre::eyre::eyre;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Xbps;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct XbpsConfig {}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct XbpsPackageOptions {}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct XbpsRepoOptions {}

impl Backend for Xbps {
    type Config = XbpsConfig;
    type PackageOptions = XbpsPackageOptions;
    type RepoOptions = XbpsRepoOptions;

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

        let stdout =
            run_command_for_stdout(["xbps-query", "--list-manual-pkgs"], Perms::Same, StdErr::Show)?;

        // Removes the package version from output
        let re = Regex::new(r"-[^-]*$")?;

        let packages = stdout
            .lines()
            .map(|line| {
                (
                    re.replace_all(line, "").to_string(),
                    Self::PackageOptions {},
                )
            })
            .collect();

        Ok(packages)
    }

    fn install_packages(
        packages: &std::collections::BTreeMap<String, Self::PackageOptions>,
        no_confirm: bool,
        _: &Self::Config,
    ) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["xbps-install", "--sync"]
                    .into_iter()
                    .chain(Some("--yes").filter(|_| no_confirm))
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
                ["xbps-remove", "--recursive"]
                    .into_iter()
                    .chain(Some("--yes").filter(|_| no_confirm))
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
                ["xbps-install", "--sync", "--update"]
                    .into_iter()
                    .chain(Some("--yes").filter(|_| no_confirm))
                    .chain(packages.iter().map(String::as_str)),
                Perms::Sudo,
            )?;
        }

        Ok(())
    }

    fn update_all_packages(no_confirm: bool, _: &Self::Config) -> Result<()> {
        let update = || {
            run_command(
                ["xbps-install", "--sync", "--update"]
                    .into_iter()
                    .chain(Some("--yes").filter(|_| no_confirm)),
                Perms::Sudo,
            )
        };

        //has to be run twice to do a full update according to the docs:
        //https://docs.voidlinux.org/xbps/index.html
        update()?;
        update()
    }

    fn clean_cache(config: &Self::Config) -> Result<()> {
        Self::version(config).map_or(Ok(()), |_| {
            run_command(
                ["xbps-remove", "--clean-cache", "--remove-orphans"],
                Perms::Sudo,
            )
        })
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
        run_command_for_stdout(["xbps-query", "--version"], Perms::Same, StdErr::Show)
    }
}
