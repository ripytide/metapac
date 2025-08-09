use std::collections::BTreeMap;
use std::collections::BTreeSet;

use color_eyre::Result;
use serde::Deserialize;
use serde::Serialize;

use crate::cmd::run_command;
use crate::cmd::run_command_for_stdout;
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Uv;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UvOptions {}

impl Backend for Uv {
    type Options = UvOptions;

    fn expand_group_packages(
        packages: BTreeMap<String, Package<Self::Options>>,
        _: &Config,
    ) -> Result<BTreeMap<String, Package<Self::Options>>> {
        Ok(packages)
    }

    fn query(config: &Config) -> Result<BTreeMap<String, Self::Options>> {
        if Self::version(config).is_err() {
            return Ok(BTreeMap::new());
        }

        let names = run_command_for_stdout(
            ["uv", "tool", "list", "--color", "never"],
            Perms::Same,
            true,
        )?
        .lines()
        .filter(|x| !x.starts_with("-"))
        .map(|x| x.split(" ").next().unwrap().to_string())
        .map(|x| (x, Self::Options {}))
        .collect();

        Ok(names)
    }

    fn install(packages: &BTreeMap<String, Self::Options>, _: bool, _: &Config) -> Result<()> {
        for package in packages.keys() {
            run_command(["uv", "tool", "install", package], Perms::Same)?;
        }

        Ok(())
    }

    fn uninstall(packages: &BTreeSet<String>, _: bool, _: &Config) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["uv", "tool", "uninstall"]
                    .into_iter()
                    .chain(packages.iter().map(String::as_str)),
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn update(packages: &BTreeSet<String>, _: bool, _: &Config) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["uv", "tool", "upgrade"]
                    .into_iter()
                    .chain(packages.iter().map(String::as_str)),
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn update_all(_: bool, _: &Config) -> Result<()> {
        run_command(["uv", "tool", "upgrade", "--all"], Perms::Same)
    }

    fn clean_cache(_: &Config) -> Result<()> {
        Ok(())
    }

    fn version(_: &Config) -> Result<String> {
        run_command_for_stdout(["uv", "--version"], Perms::Same, false)
    }
}
