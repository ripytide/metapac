use std::collections::{BTreeMap, BTreeSet};

use color_eyre::Result;
use serde::{Deserialize, Serialize};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Snap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SnapQueryInfo {}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SnapInstallOptions {}

impl Backend for Snap {
    type QueryInfo = SnapQueryInfo;
    type InstallOptions = SnapInstallOptions;

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

        let output = run_command_for_stdout(["snap", "list"], Perms::Same, false)?;
        
        // Skip the first line which is the header
        Ok(output
            .lines()
            .skip(1)
            .filter_map(|line| line.split_whitespace().next())
            .map(|name| (name.to_string(), SnapQueryInfo {}))
            .collect())
    }

    fn install_packages(
        packages: &BTreeMap<String, Self::InstallOptions>,
        no_confirm: bool,
        _: &Config,
    ) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["snap", "install"]
                    .into_iter()
                    .chain(packages.keys().map(String::as_str)),
                Perms::Sudo,
            )?;
        }

        Ok(())
    }

    fn remove_packages(packages: &BTreeSet<String>, no_confirm: bool, _: &Config) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["snap", "remove"]
                    .into_iter()
                    .chain(packages.iter().map(String::as_str)),
                Perms::Sudo,
            )?;
        }

        Ok(())
    }

    fn version(_: &Config) -> Result<String> {
        run_command_for_stdout(["snap", "--version"], Perms::Same, false)
            .map(|output| {
                output
                    .lines()
                    .next()
                    .unwrap_or_default()
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
            })
    }
}
