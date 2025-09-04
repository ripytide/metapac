use std::collections::{BTreeMap, BTreeSet};

use color_eyre::Result;
use serde::{Deserialize, Serialize};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Flatpak;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlatpakOptions {
    pub systemwide: Option<bool>,
    pub remote: Option<String>,
}

impl Backend for Flatpak {
    type Options = FlatpakOptions;

    fn invalid_package_help_text() -> String {
        String::new()
    }

    fn are_valid_packages(
        packages: &BTreeSet<String>,
        _: &Config,
    ) -> BTreeMap<String, Option<bool>> {
        packages.iter().map(|x| (x.to_string(), None)).collect()
    }

    fn query(config: &Config) -> Result<BTreeMap<String, Self::Options>> {
        if Self::version(config).is_err() {
            return Ok(BTreeMap::new());
        }

        let system_apps = run_command_for_stdout(
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
        let system_apps = system_apps.lines().map(|x| {
            (
                x.trim().to_owned(),
                Self::Options {
                    systemwide: Some(true),
                    remote: None,
                },
            )
        });

        let user_apps = run_command_for_stdout(
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
        let user_apps = user_apps.lines().map(|x| {
            (
                x.trim().to_owned(),
                Self::Options {
                    systemwide: Some(false),
                    remote: None,
                },
            )
        });

        Ok(system_apps.chain(user_apps).collect())
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
                    if options.systemwide.unwrap_or(config.flatpak.systemwide) {
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

    fn uninstall(packages: &BTreeSet<String>, no_confirm: bool, _: &Config) -> Result<()> {
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

    fn update(packages: &BTreeSet<String>, no_confirm: bool, _: &Config) -> Result<()> {
        run_command(
            ["flatpak", "update"]
                .into_iter()
                .chain(Some("--assumeyes").filter(|_| no_confirm))
                .chain(packages.iter().map(String::as_str)),
            Perms::Same,
        )
    }

    fn update_all(no_confirm: bool, _: &Config) -> Result<()> {
        run_command(
            ["flatpak", "update"]
                .into_iter()
                .chain(Some("--assumeyes").filter(|_| no_confirm)),
            Perms::Same,
        )
    }

    fn clean_cache(config: &Config) -> Result<()> {
        Self::version(config).map_or(Ok(()), |_| {
            run_command(["flatpak", "remove", "--unused"], Perms::Same)
        })
    }

    fn version(_: &Config) -> Result<String> {
        run_command_for_stdout(["flatpak", "--version"], Perms::Same, false)
    }
}
