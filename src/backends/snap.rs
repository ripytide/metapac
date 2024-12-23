use std::collections::{BTreeMap, BTreeSet};

use color_eyre::Result;
use serde::{Deserialize, Serialize};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Snap;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SnapOptions {}

impl Backend for Snap {
    type Options = SnapOptions;

    fn map_managed_packages(
        packages: BTreeMap<String, Self::Options>,
        _: &Config,
    ) -> Result<BTreeMap<String, Self::Options>> {
        Ok(packages)
    }

    fn query(config: &Config) -> Result<BTreeMap<String, Self::Options>> {
        if Self::version(config).is_err() {
            return Ok(BTreeMap::new());
        }

        let output = run_command_for_stdout(["snap", "list"], Perms::Same, false)?;

        // Skip the first line which is the header
        Ok(output
            .lines()
            .skip(1)
            .filter_map(|line| line.split_whitespace().next())
            .map(|name| (name.to_string(), Self::Options {}))
            .collect())
    }

    fn install(
        packages: &BTreeMap<String, Self::Options>,
        _: bool,
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

    fn remove(packages: &BTreeSet<String>, _: bool, _: &Config) -> Result<()> {
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

    fn clean_cache(config: &Config) -> Result<()> {
        Self::version(config).map_or(Ok(()), |_| {
            run_command(["rm", "-rf", "/var/lib/snapd/cache/*"], Perms::Sudo)
        })
    }

    fn version(_: &Config) -> Result<String> {
        run_command_for_stdout(["snap", "--version"], Perms::Same, false).map(|output| {
            output
                .lines()
                .next()
                .unwrap_or_default()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        })
    }

    fn missing(managed: Self::Options, installed: Option<Self::Options>) -> Option<Self::Options> {
        match installed {
            Some(_) => None,
            None => Some(managed),
        }
    }
}
