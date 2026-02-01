use std::collections::BTreeMap;
use std::collections::BTreeSet;

use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::cmd::run_command;
use crate::cmd::run_command_for_stdout;
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Pipx;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct PipxConfig {}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipxPackageOptions {}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipxRepoOptions {}

impl Backend for Pipx {
    type Config = PipxConfig;
    type PackageOptions = PipxPackageOptions;
    type RepoOptions = PipxRepoOptions;

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

        let names = extract_package_names(&run_command_for_stdout(
            ["pipx", "list", "--json"],
            Perms::Same,
            StdErr::Hide,
        )?)?;

        Ok(names
            .into_iter()
            .map(|x| (x, Self::PackageOptions {}))
            .collect())
    }

    fn install_packages(
        packages: &BTreeMap<String, Self::PackageOptions>,
        _: bool,
        _: &Self::Config,
    ) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["pipx", "install"]
                    .into_iter()
                    .chain(packages.keys().map(String::as_str)),
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn uninstall_packages(packages: &BTreeSet<String>, _: bool, _: &Self::Config) -> Result<()> {
        for package in packages {
            run_command(["pipx", "uninstall", package], Perms::Same)?;
        }

        Ok(())
    }

    fn update_packages(packages: &BTreeSet<String>, _: bool, _: &Self::Config) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["pipx", "update"]
                    .into_iter()
                    .chain(packages.iter().map(String::as_str)),
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn update_all_packages(_: bool, _: &Self::Config) -> Result<()> {
        run_command(["pipx", "upgrade-all"], Perms::Same)
    }

    fn clean_cache(_: &Self::Config) -> Result<()> {
        Ok(())
    }

    fn get_installed_repos(_: &Self::Config) -> Result<BTreeMap<String, Self::RepoOptions>> {
        Ok(BTreeMap::new())
    }

    fn add_repos(
        repos: &BTreeMap<String, Self::RepoOptions>,
        _: bool,
        _: &Self::Config,
    ) -> Result<()> {
        if repos.is_empty() {
            Ok(())
        } else {
            Err(eyre!("unimplemented"))
        }
    }

    fn remove_repos(repos: &BTreeSet<String>, _: bool, _: &Self::Config) -> Result<()> {
        if repos.is_empty() {
            Ok(())
        } else {
            Err(eyre!("unimplemented"))
        }
    }

    fn version(_: &Self::Config) -> Result<String> {
        run_command_for_stdout(["pipx", "--version"], Perms::Same, StdErr::Show)
    }
}

fn extract_package_names(stdout: &str) -> Result<BTreeSet<String>> {
    let value: Value = serde_json::from_str(stdout)?;

    let result = value["venvs"]
        .as_object()
        .ok_or(eyre!("getting inner json object"))?
        .iter()
        .map(|(name, _)| name.clone())
        .collect();

    Ok(result)
}
