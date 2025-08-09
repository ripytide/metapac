use std::collections::{BTreeMap, BTreeSet};

use color_eyre::Result;
use serde::{Deserialize, Serialize};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Brew;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrewOptions {}

impl Backend for Brew {
    type Options = BrewOptions;

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

        let explicit = run_command_for_stdout(
            ["brew", "list", "-1", "--quiet", "--installed-on-request"],
            Perms::Same,
            false,
        )?;

        Ok(explicit
            .lines()
            .map(|x| (x.to_string(), Self::Options {}))
            .collect())
    }

    fn install(packages: &BTreeMap<String, Self::Options>, _: bool, _: &Config) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["brew", "install"]
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
                ["brew", "remove"]
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
                ["brew", "upgrade"]
                    .into_iter()
                    .chain(packages.iter().map(String::as_str)),
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn update_all(_: bool, _: &Config) -> Result<()> {
        run_command(["brew", "upgrade"], Perms::Same)
    }

    fn clean_cache(config: &Config) -> Result<()> {
        Self::version(config).map_or(Ok(()), |_| {
            run_command(["brew", "cleanup", "--prune-prefix"], Perms::Same)
        })
    }

    fn version(_: &Config) -> Result<String> {
        run_command_for_stdout(["brew", "--version"], Perms::Same, false)
    }
}
