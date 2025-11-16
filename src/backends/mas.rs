use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Mas;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MasOptions {}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct MasConfig {}

impl Backend for Mas {
    type Options = MasOptions;
    type Config = MasConfig;

    fn invalid_package_help_text() -> String {
        String::new()
    }

    fn is_valid_package_name(_: &str) -> Option<bool> {
        None
    }

    fn get_all(_: &Self::Config) -> Result<BTreeSet<String>> {
        Err(eyre!("unimplemented"))
    }

    fn get_installed(config: &Self::Config) -> Result<BTreeMap<String, Self::Options>> {
        if Self::version(config).is_err() {
            return Ok(BTreeMap::new());
        }

        let output = run_command_for_stdout(["mas", "list"], Perms::Same, false)?;

        Ok(output
            .lines()
            .filter_map(|line| {
                // Parse lines like: "425264550   Blackmagic Disk Speed Test  (3.4.2)"
                // Extract the first field (ADAM ID) from the `mas list` output
                line.split_whitespace()
                    .next()
                    .map(|app_id| (app_id.to_string(), Self::Options {}))
            })
            .collect())
    }

    fn install(
        packages: &BTreeMap<String, Self::Options>,
        _: bool,
        _: &Self::Config,
    ) -> Result<()> {
        for (package, _) in packages {
            run_command(["mas", "install", package.as_str()], Perms::Same)?;
        }

        Ok(())
    }

    fn uninstall(packages: &BTreeSet<String>, _: bool, _: &Self::Config) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["mas", "uninstall"]
                    .into_iter()
                    .chain(packages.iter().map(String::as_str)),
                Perms::Sudo,
            )?;
        }

        Ok(())
    }

    fn update(packages: &BTreeSet<String>, _: bool, _: &Self::Config) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["mas", "upgrade"]
                    .into_iter()
                    .chain(packages.iter().map(String::as_str)),
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn update_all(_: bool, _: &Self::Config) -> Result<()> {
        run_command(["mas", "upgrade"], Perms::Same)
    }

    fn clean_cache(config: &Self::Config) -> Result<()> {
        // mas does not provide a cache cleaning mechanism
        Self::version(config).map(|_| ())
    }

    fn version(_: &Self::Config) -> Result<String> {
        run_command_for_stdout(["mas", "version"], Perms::Same, false)
    }
}
