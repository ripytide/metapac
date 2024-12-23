use std::collections::{BTreeMap, BTreeSet};

use color_eyre::Result;
use serde::{Deserialize, Serialize};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Brew;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BrewQueryInfo {}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BrewInstallOptions {}

impl Backend for Brew {
    type QueryInfo = BrewQueryInfo;
    type InstallOptions = BrewInstallOptions;

    fn map_managed_packages(
        packages: BTreeMap<String, Self::InstallOptions>,
        _: &Config,
    ) -> Result<BTreeMap<String, Self::InstallOptions>> {
        Ok(packages)
    }

    fn query_installed_packages(config: &Config) -> Result<BTreeMap<String, Self::QueryInfo>> {
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
            .map(|x| (x.to_string(), BrewQueryInfo {}))
            .collect())
    }

    fn install_packages(
        packages: &BTreeMap<String, Self::InstallOptions>,
        _: bool,
        _: &Config,
    ) -> Result<()> {
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

    fn remove_packages(packages: &BTreeSet<String>, _: bool, _: &Config) -> Result<()> {
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

    fn clean_cache(config: &Config) -> Result<()> {
        Self::version(config).map_or(Ok(()), |_| {
            run_command(["brew", "cleanup", "--prune-prefix"], Perms::Same)
        })
    }

    fn version(_: &Config) -> Result<String> {
        run_command_for_stdout(["brew", "--version"], Perms::Same, false)
    }

    fn missing(
        managed: Self::InstallOptions,
        installed: Option<Self::QueryInfo>,
    ) -> Option<Self::InstallOptions> {
        match installed {
            Some(_) => None,
            None => Some(managed),
        }
    }
}
