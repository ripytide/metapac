use std::collections::{BTreeMap, BTreeSet};

use color_eyre::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Xbps;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct XbpsOptions {}

impl Backend for Xbps {
    type Options = XbpsOptions;
    type Config = ();

    fn expand_group_packages(
        packages: BTreeMap<String, Package<Self::Options>>,
        _: &Self::Config,
    ) -> Result<BTreeMap<String, Package<Self::Options>>> {
        Ok(packages)
    }

    fn query(config: &Self::Config) -> Result<std::collections::BTreeMap<String, Self::Options>> {
        if Self::version(config).is_err() {
            return Ok(BTreeMap::new());
        }

        let stdout = run_command_for_stdout(["xbps-query", "--list-pkgs"], Perms::Same, false)?;

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
                    Self::Options {},
                )
            })
            .collect();

        Ok(packages)
    }

    fn install(
        packages: &std::collections::BTreeMap<String, Self::Options>,
        no_confirm: bool,
        _: &Self::Config,
    ) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["xbps-install", "--sync"]
                    .into_iter()
                    .chain(Some("--yes").filter(|_| no_confirm))
                    .chain(packages.keys().map(String::as_str)),
                Perms::Sudo,
            )?;
        }

        Ok(())
    }

    fn uninstall(packages: &BTreeSet<String>, no_confirm: bool, _: &Self::Config) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["xbps-remove", "--recursive"]
                    .into_iter()
                    .chain(Some("--yes").filter(|_| no_confirm))
                    .chain(packages.iter().map(String::as_str)),
                Perms::Sudo,
            )?;
        }

        Ok(())
    }

    fn update(packages: &BTreeSet<String>, no_confirm: bool, _: &Self::Config) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["xbps-install", "--sync", "--update"]
                    .into_iter()
                    .chain(Some("--yes").filter(|_| no_confirm))
                    .chain(packages.iter().map(String::as_str)),
                Perms::Sudo,
            )?;
        }

        Ok(())
    }

    fn update_all(no_confirm: bool, _: &Self::Config) -> Result<()> {
        let update = || {
            run_command(
                ["xbps-install", "--sync", "--update"]
                    .into_iter()
                    .chain(Some("--yes").filter(|_| no_confirm)),
                Perms::Sudo,
            )
        };

        //has to be run twice to do a full update according to the docs:
        //https://docs.voidlinux.org/xbps/index.html
        update()?;
        update()
    }

    fn clean_cache(config: &Self::Config) -> Result<()> {
        Self::version(config).map_or(Ok(()), |_| {
            run_command(
                ["xbps-remove", "--clean-cache", "--remove-orphans"],
                Perms::Sudo,
            )
        })
    }

    fn version(_: &Self::Config) -> Result<String> {
        run_command_for_stdout(["xbps-query", "--version"], Perms::Same, false)
    }
}
