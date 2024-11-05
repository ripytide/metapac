use std::collections::{BTreeMap, BTreeSet};

use color_eyre::Result;
use serde::{Deserialize, Serialize};

use crate::cmd::{command_found, run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct WinGet;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WinGetQueryInfo {}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WinGetInstallOptions {}

impl Backend for WinGet {
    type QueryInfo = WinGetQueryInfo;
    type InstallOptions = WinGetInstallOptions;

    fn map_managed_packages(
        packages: BTreeMap<String, Self::InstallOptions>,
        _: &Config,
    ) -> Result<BTreeMap<String, Self::InstallOptions>> {
        Ok(packages)
    }

    fn query_installed_packages(_: &Config) -> Result<BTreeMap<String, Self::QueryInfo>> {
        if !command_found("winget") {
            return Ok(BTreeMap::new());
        }

        let explicit = run_command_for_stdout(["winget", "list", "--id"], Perms::Same)?;

        Ok(explicit
            .lines()
            .map(|x| (x.to_string(), WinGetQueryInfo {}))
            .collect())
    }

    fn install_packages(
        packages: &BTreeMap<String, Self::InstallOptions>,
        _: bool,
        _: &Config,
    ) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["winget", "install", "--id", "--exact"]
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
                ["winget", "uninstall", "--id", "--exact"]
                    .into_iter()
                    .chain(packages.iter().map(String::as_str)),
                Perms::Same,
            )?;
        }

        Ok(())
    }
}
