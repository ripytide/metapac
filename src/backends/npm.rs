use std::collections::{BTreeMap, BTreeSet};

use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::backends::mise::{
    install_for, is_delegated, list_names_for_backend, uninstall_for, upgrade_all_for, upgrade_for,
};
use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Npm;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NpmOptions {}

impl Backend for Npm {
    type Options = NpmOptions;

    fn expand_group_packages(
        packages: BTreeMap<String, Package<Self::Options>>,
        _: &Config,
    ) -> Result<BTreeMap<String, Package<Self::Options>>> {
        Ok(packages)
    }

    fn query(config: &Config) -> Result<BTreeMap<String, Self::Options>> {
        // If npm is managed by mise, query via mise's installed tool list (provider == npm)
        if is_delegated(config, &AnyBackend::Npm) {
            let names = list_names_for_backend(config, &AnyBackend::Npm)?;
            return Ok(names.into_iter().map(|n| (n, NpmOptions {})).collect());
        }

        if Self::version(config).is_err() {
            return Ok(BTreeMap::new());
        }

        let stdout =
            run_command_for_stdout(["npm", "list", "--global", "--json"], Perms::Same, false)?;

        let value: Value = serde_json::from_str(&stdout)?;
        let object = value.as_object().ok_or(eyre!("json should be an object"))?;

        if !object.contains_key("dependencies") {
            return Ok(BTreeMap::new());
        }

        let names = object["dependencies"]
            .as_object()
            .ok_or(eyre!("the dependencies value should be an object"))?
            .iter()
            .map(|(name, _)| name.clone());

        Ok(names
            .into_iter()
            .map(|name| (name, NpmOptions {}))
            .collect())
    }

    fn install(packages: &BTreeMap<String, Self::Options>, _: bool, config: &Config) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        if is_delegated(config, &AnyBackend::Npm) {
            let args = BTreeMap::from_iter(packages.keys().cloned().map(|k| (k, String::new())));
            install_for(&AnyBackend::Npm, &args)?;
            return Ok(());
        }

        run_command(
            ["npm", "install", "--global"]
                .into_iter()
                .chain(packages.keys().map(String::as_str)),
            Perms::Same,
        )
    }

    fn uninstall(packages: &BTreeSet<String>, _: bool, config: &Config) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        if is_delegated(config, &AnyBackend::Npm) {
            uninstall_for(&AnyBackend::Npm, packages)?;
            return Ok(());
        }

        run_command(
            ["npm", "uninstall", "--global"]
                .into_iter()
                .chain(packages.iter().map(String::as_str)),
            Perms::Same,
        )
    }

    fn update(packages: &BTreeSet<String>, _: bool, config: &Config) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        if is_delegated(config, &AnyBackend::Npm) {
            upgrade_for(&AnyBackend::Npm, packages)?;
            return Ok(());
        }

        run_command(
            ["npm", "update", "--global"]
                .into_iter()
                .chain(packages.iter().map(String::as_str)),
            Perms::Same,
        )
    }

    fn update_all(_: bool, config: &Config) -> Result<()> {
        if is_delegated(config, &AnyBackend::Npm) {
            return upgrade_all_for(&AnyBackend::Npm);
        }
        run_command(["npm", "update", "--global"], Perms::Same)
    }

    fn clean_cache(config: &Config) -> Result<()> {
        if config.mise.manage_backends.contains(&AnyBackend::Npm) {
            // No direct mise cache clean for npm; avoid conflicting with mise internals
            return Ok(());
        }
        Self::version(config).map_or(Ok(()), |_| {
            run_command(["npm", "cache", "clean", "--force"], Perms::Same)
        })
    }

    fn version(_: &Config) -> Result<String> {
        run_command_for_stdout(["npm", "--version"], Perms::Same, false)
    }
}
