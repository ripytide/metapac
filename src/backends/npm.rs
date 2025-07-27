use std::collections::{BTreeMap, BTreeSet};

use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Npm;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NpmOptions {}

impl Backend for Npm {
    type Options = NpmOptions;

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

        let stdout =
            run_command_for_stdout(["npm", "list", "--global", "--json"], Perms::Same, false)?;

        let value: Value = serde_json::from_str(&stdout)?;
        let object = value.as_object().ok_or(eyre!("json should be an object"))?;

        if !object.contains_key("dependencies") {
            return Ok(BTreeMap::new());
        }

        let names = object["dependencies"]
            .as_object()
            .ok_or(eyre!("dependencies value should be an object"))?
            .iter()
            .map(|(name, _)| name.clone());

        Ok(names
            .into_iter()
            .map(|name| (name, NpmOptions {}))
            .collect())
    }

    fn install(packages: &BTreeMap<String, Self::Options>, _: bool, _: &Config) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["npm", "install", "--global"]
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
                ["npm", "uninstall", "--global"]
                    .into_iter()
                    .chain(packages.iter().map(String::as_str)),
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn clean_cache(config: &Config) -> Result<()> {
        Self::version(config).map_or(Ok(()), |_| {
            run_command(["npm", "cache", "clean", "--force"], Perms::Same)
        })
    }

    fn version(_: &Config) -> Result<String> {
        run_command_for_stdout(["npm", "--version"], Perms::Same, false)
    }
}
