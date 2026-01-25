use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Yarn;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct YarnConfig {}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct YarnPackageOptions {}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct YarnRepoOptions {}

impl Backend for Yarn {
    type Config = YarnConfig;
    type PackageOptions = YarnPackageOptions;
    type RepoOptions = YarnRepoOptions;

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

        //unfortunately we cant use `yarn global list` since it doesn't include packages that don't
        //have binaries, see https://github.com/yarnpkg/yarn/issues/5725
        //
        //instead we manually read the global `package.json` file
        let dir = run_command_for_stdout(["yarn", "global", "dir"], Perms::Same, StdErr::Hide)?;
        let dir = dir
            .lines()
            .next()
            .ok_or(eyre!("error getting global package directory"))?;

        let package_file = Path::new(&dir).join("package.json");

        if !package_file.exists() {
            dbg!(&package_file);
            return Ok(BTreeMap::new());
        }

        let value: Value = serde_json::from_str(&std::fs::read_to_string(package_file)?)?;
        let object = value
            .as_object()
            .ok_or(eyre!("package file should be an object"))?;

        if !object.contains_key("dependencies") {
            return Ok(BTreeMap::new());
        }

        let names = object["dependencies"]
            .as_object()
            .ok_or(eyre!("the dependencies value should be an object"))?
            .iter()
            .map(|(name, _)| name.clone());

        Ok(names
            .into_iter()
            .map(|name| (name, YarnPackageOptions {}))
            .collect())
    }

    fn install_packages(
        packages: &BTreeMap<String, Self::PackageOptions>,
        _: bool,
        _: &Self::Config,
    ) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["yarn", "global", "add"]
                    .into_iter()
                    .chain(packages.keys().map(String::as_str)),
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn uninstall_packages(packages: &BTreeSet<String>, _: bool, _: &Self::Config) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["yarn", "global", "remove"]
                    .into_iter()
                    .chain(packages.iter().map(String::as_str)),
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn update_packages(packages: &BTreeSet<String>, _: bool, _: &Self::Config) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["yarn", "global", "upgrade"]
                    .into_iter()
                    .chain(packages.iter().map(String::as_str)),
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn update_all_packages(_: bool, _: &Self::Config) -> Result<()> {
        run_command(["yarn", "global", "upgrade"], Perms::Same)
    }

    fn clean_cache(config: &Self::Config) -> Result<()> {
        Self::version(config).map_or(Ok(()), |_| {
            run_command(["yarn", "cache", "clean"], Perms::Same)
        })
    }

    fn get_installed_repos(_: &Self::Config) -> Result<BTreeMap<String, Self::RepoOptions>> {
        Ok(BTreeMap::new())
    }

    fn add_repos(repos: &BTreeMap<String, Self::RepoOptions>, _: bool, _: &Self::Config) -> Result<()> {
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
        run_command_for_stdout(["yarn", "--version"], Perms::Same, StdErr::Show)
    }
}
