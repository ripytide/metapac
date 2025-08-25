use std::collections::{BTreeMap, BTreeSet};

use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;
use crate::backends::mise::{parse_provider_and_name, query_installed_tools};

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
        if config.mise.manage_backends.contains(&AnyBackend::Npm) {
            if let Ok(tools) = query_installed_tools(config) {
                let names = tools.into_iter().filter_map(|t| {
                    parse_provider_and_name(&t)
                        .and_then(|(prov, name)| if prov == "npm" { Some(name) } else { None })
                });
                return Ok(names.map(|n| (n, NpmOptions {})).collect());
            }
            return Ok(BTreeMap::new());
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
        if packages.is_empty() { return Ok(()); }

        if config.mise.manage_backends.contains(&AnyBackend::Npm) {
            let mut args: Vec<String> = vec!["mise".into(), "install".into()];
            args.extend(packages.keys().map(|k| format!("npm:{k}")));
            run_command(args, Perms::Same)?;
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
        if packages.is_empty() { return Ok(()); }

        if config.mise.manage_backends.contains(&AnyBackend::Npm) {
            for package in packages {
                run_command(["mise", "uninstall", &format!("npm:{package}")], Perms::Same)?;
            }
            return Ok(());
        }

        run_command(["npm", "uninstall", "--global"].into_iter().chain(packages.iter().map(String::as_str)), Perms::Same)
    }

    fn update(packages: &BTreeSet<String>, _: bool, config: &Config) -> Result<()> {
        if packages.is_empty() { return Ok(()); }

        if config.mise.manage_backends.contains(&AnyBackend::Npm) {
            // Upgrade only the provided npm tools via mise
            for package in packages {
                run_command(["mise", "upgrade", &format!("npm:{package}")], Perms::Same)?;
            }
            return Ok(());
        }

        run_command(["npm", "update", "--global"].into_iter().chain(packages.iter().map(String::as_str)), Perms::Same)
    }

    fn update_all(_: bool, config: &Config) -> Result<()> {
        if config.mise.manage_backends.contains(&AnyBackend::Npm) {
            // Scope upgrades to npm tools only
            return run_command(["mise", "upgrade", "npm:*"], Perms::Same);
        }
        run_command(["npm", "update", "--global"], Perms::Same)
    }

    fn clean_cache(config: &Config) -> Result<()> {
        if config.mise.manage_backends.contains(&AnyBackend::Npm) {
            // No direct mise cache clean for npm; avoid conflicting with mise internals
            return Ok(());
        }
        Self::version(config).map_or(Ok(()), |_| run_command(["npm", "cache", "clean", "--force"], Perms::Same))
    }

    fn version(_: &Config) -> Result<String> {
        run_command_for_stdout(["npm", "--version"], Perms::Same, false)
    }
}
