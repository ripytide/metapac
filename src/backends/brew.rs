use std::collections::{BTreeMap, BTreeSet};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Brew;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrewOptions {
    quarantine: Option<bool>,
}

#[serde_inline_default]
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrewConfig {
    #[serde_inline_default(FlatpakConfig::default().systemwide)]
    quarantine: bool,
}
impl Default for BrewConfig {
    fn default() -> Self {
        Self { quarantine: true }
    }
}

impl Backend for Brew {
    type Options = BrewOptions;
    type Config = BrewConfig;

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

        let formulae = run_command_for_stdout(
            ["brew", "list", "-1", "--quiet", "--installed-on-request"],
            Perms::Same,
            false,
        )?;

        let casks = run_command_for_stdout(
            ["brew", "list", "-1", "--cask", "--quiet"],
            Perms::Same,
            false,
        )?;

        Ok(formulae
            .lines()
            .chain(casks.lines())
            .map(|x| (x.to_string(), Self::Options { quarantine: None }))
            .collect())
    }

    fn install(
        packages: &BTreeMap<String, Self::Options>,
        _: bool,
        config: &Self::Config,
    ) -> Result<()> {
        for (package, options) in packages {
            run_command(
                [
                    "brew",
                    "install",
                    if options.quarantine.unwrap_or(config.quarantine) {
                        "--quarantine"
                    } else {
                        "--no-quarantine"
                    },
                    package.as_str(),
                ]
                .into_iter(),
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn uninstall(packages: &BTreeSet<String>, _: bool, _: &Self::Config) -> Result<()> {
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

    fn update(packages: &BTreeSet<String>, _: bool, _: &Self::Config) -> Result<()> {
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

    fn update_all(_: bool, _: &Self::Config) -> Result<()> {
        run_command(["brew", "upgrade"], Perms::Same)
    }

    fn clean_cache(config: &Self::Config) -> Result<()> {
        Self::version(config).map_or(Ok(()), |_| {
            run_command(["brew", "cleanup", "--prune-prefix"], Perms::Same)
        })
    }

    fn version(_: &Self::Config) -> Result<String> {
        run_command_for_stdout(["brew", "--version"], Perms::Same, false)
    }
}
