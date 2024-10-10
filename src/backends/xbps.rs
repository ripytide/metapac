use std::collections::BTreeMap;
use std::process::Command;

use color_eyre::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::cmd::{command_found, run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Xbps;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct XbpsQueryInfo {}
impl PossibleQueryInfo for XbpsQueryInfo {
    fn explicit(&self) -> Option<bool> {
        None
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct XbpsInstallOptions {}

#[derive(Debug, Clone)]
pub struct XbpsModificationOptions {
    make_implicit: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct XbpsRemoveOptions {}

impl Backend for Xbps {
    type QueryInfo = XbpsQueryInfo;
    type InstallOptions = XbpsInstallOptions;
    type ModificationOptions = XbpsModificationOptions;
    type RemoveOptions = XbpsRemoveOptions;

    fn query_installed_packages(
        _: &Config,
    ) -> Result<std::collections::BTreeMap<String, Self::QueryInfo>> {
        if !command_found("xbps-query") {
            return Ok(BTreeMap::new());
        }

        let mut cmd = Command::new("xbps-query");
        cmd.args(["-l"]);
        let stdout = run_command_for_stdout(["xbps-query", "-l"], Perms::Same)?;

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
        run_command(
            ["xbps-install", "-S"]
                .into_iter()
                .chain(Some("-y").filter(|_| no_confirm))
                .chain(packages.keys().map(String::as_str)),
            Perms::AsRoot,
        )
    }

    fn remove_packages(
        packages: &std::collections::BTreeMap<String, Self::RemoveOptions>,
        no_confirm: bool,
        _: &Config,
    ) -> Result<()> {
        run_command(
            ["xbps-remove", "-R"]
                .into_iter()
                .chain(Some("-y").filter(|_| no_confirm))
                .chain(packages.keys().map(String::as_str)),
            Perms::AsRoot,
        )
    }

    fn modify_packages(
        packages: &std::collections::BTreeMap<String, Self::ModificationOptions>,
        _: &Config,
    ) -> Result<()> {
        run_command(
            ["xbps-pkgdb", "-m", "auto"].into_iter().chain(
                packages
                    .iter()
                    .filter(|(_, m)| m.make_implicit)
                    .map(|(p, _)| p.as_str()),
            ),
            Perms::AsRoot,
        )
    }
}
