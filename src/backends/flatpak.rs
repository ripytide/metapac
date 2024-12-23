use std::collections::{BTreeMap, BTreeSet};

use color_eyre::Result;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Flatpak;

#[derive(Debug, Clone)]
pub struct FlatpakQueryInfo {
    pub systemwide: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FlatpakInstallOptions {
    pub remote: String,
}

impl Backend for Flatpak {
    type QueryInfo = FlatpakQueryInfo;
    type InstallOptions = FlatpakInstallOptions;

    fn map_managed_packages(
        packages: BTreeMap<String, Self::InstallOptions>,
        _: &Config,
    ) -> Result<BTreeMap<String, Self::InstallOptions>> {
        Ok(packages)
    }

    fn query_installed_packages(config: &Config) -> Result<BTreeMap<String, Self::QueryInfo>> {
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
        let sys_explicit = sys_explicit_out
            .lines()
            .map(|x| (x.trim().to_owned(), FlatpakQueryInfo { systemwide: true }));

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
        let user_explicit = user_explicit_out
            .lines()
            .map(|x| (x.trim().to_owned(), FlatpakQueryInfo { systemwide: false }));

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
                    FlatpakQueryInfo { systemwide: true },
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
                    FlatpakQueryInfo { systemwide: false },
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

    fn install_packages(
        packages: &BTreeMap<String, Self::InstallOptions>,
        no_confirm: bool,
        config: &Config,
    ) -> Result<()> {
        let mut no_remotes = Vec::new();
        let mut remote_packages = BTreeMap::new();
        for (package, remote) in packages {
            if remote.remote.is_empty() {
                no_remotes.push(package);
            } else {
                remote_packages.insert(package, remote);
            }
        }

        if !no_remotes.is_empty() {
            run_command(
                [
                    "flatpak",
                    "install",
                    if config.flatpak_systemwide {
                        "--system"
                    } else {
                        "--user"
                    },
                ]
                .into_iter()
                .chain(Some("--assumeyes").filter(|_| no_confirm))
                .chain(no_remotes.into_iter().map(String::as_str)),
                Perms::Same,
            )?;
        }

        for (package, remote) in remote_packages {
            run_command(
                [
                    "flatpak",
                    "install",
                    if config.flatpak_systemwide {
                        "--system"
                    } else {
                        "--user"
                    },
                ]
                .into_iter()
                .chain(Some("--assumeyes").filter(|_| no_confirm))
                .chain([remote.remote.as_str()])
                .chain([package.as_str()]),
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn remove_packages(
        packages: &BTreeSet<String>,
        no_confirm: bool,
        config: &Config,
    ) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                [
                    "flatpak",
                    "uninstall",
                    if config.flatpak_systemwide {
                        "--system"
                    } else {
                        "--user"
                    },
                ]
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

    fn missing(
        managed: Self::InstallOptions,
        installed: Option<Self::QueryInfo>,
    ) -> Option<Self::InstallOptions> {
        match installed {
            Some(_) => None,
            None => Some(managed),
        }
    }
}
