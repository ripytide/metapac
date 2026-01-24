use std::collections::{BTreeMap, BTreeSet};

use color_eyre::Result;
use color_eyre::eyre::eyre;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Scoop;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ScoopConfig {}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScoopPackageOptions {}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScoopRepoOptions {}

impl Backend for Scoop {
    type Config = ScoopConfig;
    type PackageOptions = ScoopPackageOptions;
    type RepoOptions = ScoopRepoOptions;

    fn invalid_package_help_text() -> String {
        indoc::formatdoc! {"
            A scoop package may be invalid due to one of the following issues:
                - the package name does not use the explicit \"bucket/package\" format which is
                  required by metapac in order to unambiguously match installed packages with those
                  declared in your group files
        "}
    }

    fn is_valid_package_name(package: &str) -> Option<bool> {
        Some(
            Regex::new("[a-zA-Z0-9]+/[a-zA-Z0-9]+")
                .unwrap()
                .is_match(package),
        )
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

        let output = run_command_for_stdout(["scoop.cmd", "list"], Perms::Same, false)?;
        let lines = output.lines().collect::<Vec<_>>();

        let mut packages = BTreeMap::new();
        //ignore the first four and the last lines
        for line in lines.into_iter().skip(4).rev().skip(1).rev() {
            let parts = line.split_whitespace().collect::<Vec<_>>();

            let name = parts.first().ok_or(eyre!("unexpected output"))?;
            let bucket = parts.get(2).ok_or(eyre!("unexpected output"))?;

            packages.insert(format!("{bucket}/{name}"), Self::PackageOptions {});
        }

        Ok(packages)
    }

    fn install_packages(
        packages: &BTreeMap<String, Self::PackageOptions>,
        _: bool,
        _: &Self::Config,
    ) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["scoop.cmd", "install"]
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
                ["scoop.cmd", "uninstall", "--purge"]
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
                ["scoop.cmd", "update"]
                    .into_iter()
                    .chain(packages.iter().map(String::as_str)),
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn update_all_packages(_: bool, _: &Self::Config) -> Result<()> {
        run_command(["scoop.cmd", "update", "--all"], Perms::Same)
    }

    fn clean_cache(_: &Self::Config) -> Result<()> {
        run_command(["scoop.cmd", "cache", "rm", "--all"], Perms::Same)?;
        run_command(["scoop.cmd", "cleanup", "--all", "--cache"], Perms::Same)
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
        let output = run_command_for_stdout(["scoop.cmd", "--version"], Perms::Same, false)?;

        Ok(output.lines().nth(1).unwrap().to_string())
    }
}
