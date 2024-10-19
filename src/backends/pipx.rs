use std::collections::BTreeMap;
use std::collections::BTreeSet;

use color_eyre::eyre::eyre;
use color_eyre::Result;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::cmd::command_found;
use crate::cmd::run_command;
use crate::cmd::run_command_for_stdout;
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Pipx;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PipxQueryOptions {}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PipxInstallOptions {}

impl Backend for Pipx {
    type QueryInfo = PipxQueryOptions;
    type InstallOptions = PipxInstallOptions;

    fn query_installed_packages(_: &Config) -> Result<BTreeMap<String, Self::QueryInfo>> {
        if !command_found("pipx") {
            return Ok(BTreeMap::new());
        }

        let names = extract_package_names(run_command_for_stdout(
            ["pipx", "list", "--json"],
            Perms::Same,
        )?)?;

        Ok(names
            .into_iter()
            .map(|x| (x, PipxQueryOptions {}))
            .collect())
    }

    fn install_packages(
        packages: &BTreeMap<String, Self::InstallOptions>,
        _: bool,
        _: &Config,
    ) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["pipx", "install"]
                    .into_iter()
                    .chain(packages.keys().map(String::as_str)),
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn remove_packages(
        packages: &BTreeSet<String>,
        _: bool,
        _: &Config,
    ) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["pipx", "uninstall"]
                    .into_iter()
                    .chain(packages.iter().map(String::as_str)),
                Perms::Same,
            )?;
        }

        Ok(())
    }
}

fn extract_package_names(stdout: String) -> Result<BTreeSet<String>> {
    let value: Value = serde_json::from_str(&stdout)?;

    let result = value["venvs"]
        .as_object()
        .ok_or(eyre!("getting inner json object"))?
        .iter()
        .map(|(name, _)| name.clone())
        .collect();

    Ok(result)
}
