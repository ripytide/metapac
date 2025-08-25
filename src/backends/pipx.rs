use std::collections::BTreeMap;
use std::collections::BTreeSet;

use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::backends::mise::{
    install_for, is_delegated, list_names_for_backend, uninstall_for, upgrade_all_for, upgrade_for,
};
use crate::cmd::run_command;
use crate::cmd::run_command_for_stdout;
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Pipx;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipxOptions {}

impl Backend for Pipx {
    type Options = PipxOptions;

    fn expand_group_packages(
        packages: BTreeMap<String, Package<Self::Options>>,
        _: &Config,
    ) -> Result<BTreeMap<String, Package<Self::Options>>> {
        Ok(packages)
    }

    fn query(config: &Config) -> Result<BTreeMap<String, Self::Options>> {
        // If pipx is managed by mise, query via mise's installed tool list (provider == pipx)
        if is_delegated(config, &AnyBackend::Pipx) {
            let names = list_names_for_backend(config, &AnyBackend::Pipx)?;
            return Ok(names.into_iter().map(|x| (x, Self::Options {})).collect());
        }

        if Self::version(config).is_err() {
            return Ok(BTreeMap::new());
        }

        let names = extract_package_names(run_command_for_stdout(
            ["pipx", "list", "--json"],
            Perms::Same,
            true,
        )?)?;

        Ok(names.into_iter().map(|x| (x, Self::Options {})).collect())
    }

    fn install(packages: &BTreeMap<String, Self::Options>, _: bool, config: &Config) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        if is_delegated(config, &AnyBackend::Pipx) {
            let args = BTreeMap::from_iter(packages.keys().cloned().map(|k| (k, String::new())));
            install_for(&AnyBackend::Pipx, &args)?;
            return Ok(());
        }

        run_command(
            ["pipx", "install"]
                .into_iter()
                .chain(packages.keys().map(String::as_str)),
            Perms::Same,
        )
    }

    fn uninstall(packages: &BTreeSet<String>, _: bool, config: &Config) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        if is_delegated(config, &AnyBackend::Pipx) {
            uninstall_for(&AnyBackend::Pipx, packages)?;
            return Ok(());
        }

        for package in packages {
            run_command(["pipx", "uninstall", package], Perms::Same)?;
        }
        Ok(())
    }

    fn update(packages: &BTreeSet<String>, _: bool, config: &Config) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        if is_delegated(config, &AnyBackend::Pipx) {
            upgrade_for(&AnyBackend::Pipx, packages)?;
            return Ok(());
        }

        run_command(
            ["pipx", "update"]
                .into_iter()
                .chain(packages.iter().map(String::as_str)),
            Perms::Same,
        )
    }

    fn update_all(_: bool, config: &Config) -> Result<()> {
        if is_delegated(config, &AnyBackend::Pipx) {
            return upgrade_all_for(&AnyBackend::Pipx);
        }
        run_command(["pipx", "update-all"], Perms::Same)
    }

    fn clean_cache(config: &Config) -> Result<()> {
        if config.mise.manage_backends.contains(&AnyBackend::Pipx) {
            // No direct cache clean via mise
            return Ok(());
        }
        Ok(())
    }

    fn version(_: &Config) -> Result<String> {
        run_command_for_stdout(["pipx", "--version"], Perms::Same, false)
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
