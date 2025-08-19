use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::cmd::{run_command, run_command_for_stdout};
use crate::config::YarnConfig;
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Yarn;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct YarnOptions {}

impl Backend for Yarn {
    type Options = YarnOptions;
    type Config = YarnConfig;

    fn expand_group_packages(
        packages: BTreeMap<String, Package<Self::Options>>,
        _: &Self::Config,
    ) -> Result<BTreeMap<String, Package<Self::Options>>> {
        Ok(packages)
    }

    fn query(config: &Self::Config) -> Result<BTreeMap<String, Self::Options>> {
        if Self::version(config).is_err() {
            return Ok(BTreeMap::new());
        }

        //unfortunately we cant use `yarn global list` since it doesn't include packages that don't
        //have binaries, see https://github.com/yarnpkg/yarn/issues/5725
        //
        //instead we manually read the global `package.json` file
        let dir = run_command_for_stdout(["yarn", "global", "dir"], Perms::Same, true)?;
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
            .map(|name| (name, YarnOptions {}))
            .collect())
    }

    fn install(
        packages: &BTreeMap<String, Self::Options>,
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

    fn uninstall(packages: &BTreeSet<String>, _: bool, _: &Self::Config) -> Result<()> {
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

    fn update(packages: &BTreeSet<String>, _: bool, _: &Self::Config) -> Result<()> {
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

    fn update_all(_: bool, _: &Self::Config) -> Result<()> {
        run_command(["yarn", "global", "upgrade"], Perms::Same)
    }

    fn clean_cache(config: &Self::Config) -> Result<()> {
        Self::version(config).map_or(Ok(()), |_| {
            run_command(["yarn", "cache", "clean"], Perms::Same)
        })
    }

    fn version(_: &Self::Config) -> Result<String> {
        run_command_for_stdout(["yarn", "--version"], Perms::Same, false)
    }
}
