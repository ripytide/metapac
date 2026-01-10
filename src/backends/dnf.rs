use std::collections::{BTreeMap, BTreeSet};

use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Dnf;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DnfOptions {}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct DnfConfig {}

impl Backend for Dnf {
    type Options = DnfOptions;
    type Config = DnfConfig;

    fn invalid_package_help_text() -> String {
        String::new()
    }

    fn is_valid_package_name(_: &str) -> Option<bool> {
        None
    }

    fn get_all(_: &Self::Config) -> Result<BTreeSet<String>> {
        Err(eyre!("unimplemented"))
    }

    fn get_installed(config: &Self::Config) -> Result<BTreeMap<String, Self::Options>> {
        if Self::version(config).is_err() {
            return Ok(BTreeMap::new());
        }

        let packages = run_command_for_stdout(
            [
                "dnf",
                "repoquery",
                "--userinstalled",
                "--queryformat",
                "%{name}\n",
            ],
            Perms::Same,
            false,
        )?;

        Ok(packages
            .lines()
            .map(|x| (x.to_string(), Self::Options {}))
            .collect())
    }

    fn install(
        packages: &BTreeMap<String, Self::Options>,
        no_confirm: bool,
        _: &Self::Config,
    ) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["dnf", "install"]
                    .into_iter()
                    .chain(Some("--assumeyes").filter(|_| no_confirm))
                    .chain(packages.keys().map(String::as_str)),
                Perms::Sudo,
            )?;
        }

        Ok(())
    }

    fn uninstall(packages: &BTreeSet<String>, no_confirm: bool, _: &Self::Config) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["dnf", "remove"]
                    .into_iter()
                    .chain(Some("--assumeyes").filter(|_| no_confirm))
                    .chain(packages.iter().map(String::as_str)),
                Perms::Sudo,
            )?;
        }

        Ok(())
    }

    fn update(packages: &BTreeSet<String>, no_confirm: bool, _: &Self::Config) -> Result<()> {
        run_command(
            ["dnf", "upgrade"]
                .into_iter()
                .chain(Some("--assumeyes").filter(|_| no_confirm))
                .chain(packages.iter().map(String::as_str)),
            Perms::Sudo,
        )
    }

    fn update_all(no_confirm: bool, _: &Self::Config) -> Result<()> {
        run_command(
            ["dnf", "upgrade"]
                .into_iter()
                .chain(Some("--assumeyes").filter(|_| no_confirm)),
            Perms::Sudo,
        )
    }

    fn clean_cache(config: &Self::Config) -> Result<()> {
        Self::version(config).map_or(Ok(()), |_| {
            run_command(["dnf", "clean", "all"], Perms::Same)
        })
    }

    fn version(_: &Self::Config) -> Result<String> {
        run_command_for_stdout(["dnf", "--version"], Perms::Same, false)
    }
}
