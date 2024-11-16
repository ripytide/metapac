use std::collections::{BTreeMap, BTreeSet};
use std::process::Command;

use color_eyre::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Xbps;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct XbpsQueryInfo {}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct XbpsInstallOptions {}

impl Backend for Xbps {
    type QueryInfo = XbpsQueryInfo;
    type InstallOptions = XbpsInstallOptions;

    fn map_managed_packages(
        packages: BTreeMap<String, Self::InstallOptions>,
        _: &Config,
    ) -> Result<BTreeMap<String, Self::InstallOptions>> {
        Ok(packages)
    }

    fn query_installed_packages(
        config: &Config,
    ) -> Result<std::collections::BTreeMap<String, Self::QueryInfo>> {
        if Self::version(config).is_ok() {
            return Ok(BTreeMap::new());
        }

        let mut cmd = Command::new("xbps-query");
        cmd.args(["-l"]);
        let stdout = run_command_for_stdout(["xbps-query", "-l"], Perms::Same, false)?;

        // Removes the package status and description from output
        let re1 = Regex::new(r"^ii |^uu |^hr |^\?\? | .*")?;
        // Removes the package version from output
        let re2 = Regex::new(r"-[^-]*$")?;

        let packages = stdout
            .lines()
            .map(|line| {
                let mid_result = re1.replace_all(line, "");

                (
                    re2.replace_all(&mid_result, "").to_string(),
                    XbpsQueryInfo {},
                )
            })
            .collect();

        Ok(packages)
    }

    fn install_packages(
        packages: &std::collections::BTreeMap<String, Self::InstallOptions>,
        no_confirm: bool,
        _: &Config,
    ) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["xbps-install", "-S"]
                    .into_iter()
                    .chain(Some("-y").filter(|_| no_confirm))
                    .chain(packages.keys().map(String::as_str)),
                Perms::Sudo,
            )?;
        }

        Ok(())
    }

    fn remove_packages(packages: &BTreeSet<String>, no_confirm: bool, _: &Config) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["xbps-remove", "-R"]
                    .into_iter()
                    .chain(Some("-y").filter(|_| no_confirm))
                    .chain(packages.iter().map(String::as_str)),
                Perms::Sudo,
            )?;
        }

        Ok(())
    }

    fn version(_: &Config) -> Result<String> {
        run_command_for_stdout(["xbps-query", "--version"], Perms::Same, false)
    }
}
