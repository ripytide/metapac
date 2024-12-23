use std::collections::{BTreeMap, BTreeSet};

use color_eyre::Result;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Flatpak;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FlatpakOptions {
    pub systemwide: Option<bool>,
    pub remote: Option<String>,
}

impl Backend for Flatpak {
    type Options = FlatpakOptions;

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

        let sys_explicit_out = run_command_for_stdout(
            [
                "flatpak",
                "list",
                "--system",
                "--app",
                "--columns=application",
            ],
            Perms::Same,
            false,
        )?;
        let sys_explicit = sys_explicit_out.lines().map(|x| {
            (
                x.trim().to_owned(),
                Self::Options {
                    systemwide: Some(true),
                    remote: None,
                },
            )
        });

        let user_explicit_out = run_command_for_stdout(
            [
                "flatpak",
                "list",
                "--user",
                "--app",
                "--columns=application",
            ],
            Perms::Same,
            false,
        )?;
        let user_explicit = user_explicit_out.lines().map(|x| {
            (
                x.trim().to_owned(),
                Self::Options {
                    systemwide: Some(false),
                    remote: None,
                },
            )
        });

        let sys_explicit_runtimes_installed = run_command_for_stdout(
            [
                "flatpak",
                "list",
                "--system",
                "--runtime",
                "--columns=application",
            ],
            Perms::Same,
            false,
        )?;
        let sys_explicit_runtimes_out =
            run_command_for_stdout(["flatpak", "pin", "--system"], Perms::Same, false)?;
        let sys_explicit_runtimes = sys_explicit_runtimes_out
            .lines()
            .map(|x| {
                (
                    x.trim().split('/').nth(1).unwrap().to_owned(),
                    Self::Options {
                        systemwide: Some(true),
                        remote: None,
                    },
                )
            })
            .filter(|(runtime, _)| {
                sys_explicit_runtimes_installed
                    .lines()
                    .map(|x| x.trim())
                    .contains(&runtime.as_str())
            });

        let user_explicit_runtimes_installed = run_command_for_stdout(
            [
                "flatpak",
                "list",
                "--user",
                "--runtime",
                "--columns=application",
            ],
            Perms::Same,
            false,
        )?;
        let user_explicit_runtimes_out =
            run_command_for_stdout(["flatpak", "pin", "--user"], Perms::Same, false)?;
        let user_explicit_runtimes = user_explicit_runtimes_out
            .lines()
            .map(|x| {
                (
                    x.trim().split('/').nth(1).unwrap().to_owned(),
                    Self::Options {
                        systemwide: Some(false),
                        remote: None,
                    },
                )
            })
            .filter(|(runtime, _)| {
                user_explicit_runtimes_installed
                    .lines()
                    .map(|x| x.trim())
                    .contains(&runtime.as_str())
            });

        let all = sys_explicit
            .chain(user_explicit)
            .chain(sys_explicit_runtimes)
            .chain(user_explicit_runtimes)
            .collect();

        Ok(all)
    }

    fn install(
        packages: &BTreeMap<String, Self::Options>,
        no_confirm: bool,
        config: &Config,
    ) -> Result<()> {
        for (package, options) in packages {
            run_command(
                [
                    "flatpak",
                    "install",
                    if options
                        .systemwide
                        .unwrap_or(config.flatpak_default_systemwide)
                    {
                        "--system"
                    } else {
                        "--user"
                    },
                ]
                .into_iter()
                .chain(Some("--assumeyes").filter(|_| no_confirm))
                .chain(options.remote.as_deref())
                .chain([package.as_str()]),
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn remove(packages: &BTreeSet<String>, no_confirm: bool, _: &Config) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["flatpak", "uninstall"]
                    .into_iter()
                    .chain(Some("--assumeyes").filter(|_| no_confirm))
                    .chain(packages.iter().map(String::as_str)),
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn clean_cache(config: &Config) -> Result<()> {
        Self::version(config).map_or(Ok(()), |_| {
            run_command(["flatpak", "remove", "--unused"], Perms::Same)
        })
    }

    fn version(_: &Config) -> Result<String> {
        run_command_for_stdout(["flatpak", "--version"], Perms::Same, false)
    }

    fn missing(managed: Self::Options, installed: Option<Self::Options>) -> Option<Self::Options> {
        match installed {
            Some(_) => None,
            None => Some(managed),
        }
    }
}
