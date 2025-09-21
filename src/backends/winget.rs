use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;

use color_eyre::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct WinGet;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WinGetOptions {}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct WinGetConfig {}

impl Backend for WinGet {
    type Options = WinGetOptions;
    type Config = WinGetConfig;

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

        //TODO: refactor if https://github.com/microsoft/winget-cli/issues/184 or https://github.com/microsoft/winget-cli/issues/4267 are ever fixed
        let mut tempfile = tempfile::NamedTempFile::new()?;
        let _ = run_command_for_stdout(
            [
                "winget",
                "export",
                "--nowarn",
                tempfile.path().to_str().unwrap(),
            ],
            Perms::Same,
            false,
        )?;

        let mut export = String::new();
        tempfile.read_to_string(&mut export)?;

        let export: Value = serde_json::from_str(&export)?;

        Ok(export["Sources"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|x| x["Packages"].as_array().unwrap())
            .map(|x| {
                (
                    x["PackageIdentifier"].as_str().unwrap().to_string(),
                    Self::Options {},
                )
            })
            .collect())
    }

    fn install(
        packages: &BTreeMap<String, Self::Options>,
        _: bool,
        _: &Self::Config,
    ) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["winget", "install"]
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
                ["winget", "uninstall"]
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
                ["winget", "update"]
                    .into_iter()
                    .chain(packages.iter().map(String::as_str)),
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn update_all(_: bool, _: &Self::Config) -> Result<()> {
        run_command(["winget", "update"], Perms::Same)
    }

    // currently there is no way to do it for winget, see
    // https://github.com/microsoft/winget-cli/issues/343
    fn clean_cache(_: &Self::Config) -> Result<()> {
        Ok(())
    }

    fn version(_: &Self::Config) -> Result<String> {
        run_command_for_stdout(["winget", "--version"], Perms::Same, false)
    }
}
