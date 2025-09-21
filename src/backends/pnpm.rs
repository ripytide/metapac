use std::collections::{BTreeMap, BTreeSet};

use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Pnpm;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PnpmOptions {}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct PnpmConfig {}

impl Backend for Pnpm {
    type Options = PnpmOptions;
    type Config = PnpmConfig;

    fn invalid_package_help_text() -> String {
        String::new()
    }

    fn are_valid_packages(
        packages: &BTreeSet<String>,
        _: &Config,
    ) -> BTreeMap<String, Option<bool>> {
        packages.iter().map(|x| (x.to_string(), None)).collect()
    }

    fn get_installed(config: &Self::Config) -> Result<BTreeMap<String, Self::Options>> {
        if Self::version(config).is_err() {
            return Ok(BTreeMap::new());
        }

        let stdout =
            run_command_for_stdout(["pnpm", "list", "--global", "--json"], Perms::Same, false)?;

        let value: Value = serde_json::from_str(&stdout)?;
        let first_object = value.as_array().ok_or(eyre!("json should be an array"))?[0]
            .as_object()
            .ok_or(eyre!("json should be an object"))?;

        if !first_object.contains_key("dependencies") {
            return Ok(BTreeMap::new());
        }

        let names = first_object["dependencies"]
            .as_object()
            .ok_or(eyre!("the dependencies value should be an object"))?
            .iter()
            .map(|(name, _)| name.clone());

        Ok(names
            .into_iter()
            .map(|name| (name, PnpmOptions {}))
            .collect())
    }

    fn install(
        packages: &BTreeMap<String, Self::Options>,
        _: bool,
        _: &Self::Config,
    ) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["pnpm", "install", "--global"]
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
                ["pnpm", "uninstall", "--global"]
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
                ["pnpm", "update", "--global"]
                    .into_iter()
                    .chain(packages.iter().map(String::as_str)),
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn update_all(_: bool, _: &Self::Config) -> Result<()> {
        run_command(["pnpm", "update", "--global"], Perms::Same)
    }

    fn clean_cache(config: &Self::Config) -> Result<()> {
        Self::version(config).map_or(Ok(()), |_| {
            run_command(["pnpm", "store", "prune"], Perms::Same)
        })
    }

    fn version(_: &Self::Config) -> Result<String> {
        run_command_for_stdout(["pnpm", "--version"], Perms::Same, false)
    }
}
