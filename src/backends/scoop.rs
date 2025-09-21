use std::collections::{BTreeMap, BTreeSet};

use color_eyre::Result;
use color_eyre::eyre::eyre;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Scoop;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScoopGetOptions {}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ScoopConfig {}

impl Backend for Scoop {
    type Options = ScoopGetOptions;
    type Config = ScoopConfig;

    fn invalid_package_help_text() -> String {
        indoc::formatdoc! {"
            A scoop package may be invalid due to one of the following issues:
                - the package name has a typo as written in your group files
                - the package name does not use the explicit \"bucket/package\" format which is
                  required by metapac in order to unambiguously match installed packages with those
                  declared in your group files
        "}
    }

    fn is_valid_package_name(package: &str) -> Option<bool> {
        Some(Regex::new("[a-zA-Z0-9]+/[a-zA-Z0-9]+").unwrap().is_match(package))
    }

    fn get_all(_: &Self::Config) -> Result<BTreeSet<String>> {
        Err(eyre!("unimplemented"))
    }

    fn get_installed(config: &Self::Config) -> Result<BTreeMap<String, Self::Options>> {
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

            packages.insert(format!("{bucket}/{name}"), Self::Options {});
        }

        Ok(packages)
    }

    fn install(
        packages: &BTreeMap<String, Self::Options>,
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

    fn uninstall(packages: &BTreeSet<String>, _: bool, _: &Self::Config) -> Result<()> {
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

    fn update(packages: &BTreeSet<String>, _: bool, _: &Self::Config) -> Result<()> {
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

    fn update_all(_: bool, _: &Self::Config) -> Result<()> {
        run_command(["scoop.cmd", "update", "--all"], Perms::Same)
    }

    fn clean_cache(_: &Self::Config) -> Result<()> {
        run_command(["scoop.cmd", "cache", "rm", "--all"], Perms::Same)?;
        run_command(["scoop.cmd", "cleanup", "--all", "--cache"], Perms::Same)
    }

    fn version(_: &Self::Config) -> Result<String> {
        let output = run_command_for_stdout(["scoop.cmd", "--version"], Perms::Same, false)?;

        Ok(output.lines().nth(1).unwrap().to_string())
    }
}
