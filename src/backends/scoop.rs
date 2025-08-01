use std::collections::{BTreeMap, BTreeSet};

use color_eyre::Result;
use serde::{Deserialize, Serialize};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Scoop;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScoopGetOptions {}

impl Backend for Scoop {
    type Options = ScoopGetOptions;

    fn map_required(
        packages: BTreeMap<String, Package<Self::Options>>,
        _: &Config,
    ) -> Result<BTreeMap<String, Package<Self::Options>>> {
        Ok(packages)
    }

    fn query(config: &Config) -> Result<BTreeMap<String, Self::Options>> {
        if Self::version(config).is_err() {
            return Ok(BTreeMap::new());
        }

        let output = run_command_for_stdout(
            [
                "scoop.cmd",
                "list",
            ],
            Perms::Same,
            false,
        )?;

        Ok(output
            .lines()
            .skip(4)
            .map(|x| {
                (
                    x.split(" ").next().unwrap().to_string(),
                    Self::Options {},
                )
            })
            .collect())
    }

    fn install(packages: &BTreeMap<String, Self::Options>, _: bool, _: &Config) -> Result<()> {
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

    fn uninstall(packages: &BTreeSet<String>, _: bool, _: &Config) -> Result<()> {
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

    fn clean_cache(_: &Config) -> Result<()> {
        run_command(["scoop.cmd", "cache", "rm", "--all"], Perms::Same)?;
        run_command(["scoop.cmd", "cleanup", "--all", "--cache"], Perms::Same)
    }

    fn version(_: &Config) -> Result<String> {
        let output = run_command_for_stdout(
            [
                "scoop.cmd",
                "--version",
            ],
            Perms::Same,
            false,
        )?;

        Ok(output.lines().nth(1).unwrap().to_string())
    }
}
