use std::collections::{BTreeMap, BTreeSet};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Flatpak;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlatpakOptions {
    pub systemwide: Option<bool>,
    pub remote: Option<String>,
}

#[serde_inline_default]
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlatpakConfig {
    #[serde_inline_default(FlatpakConfig::default().systemwide)]
    pub systemwide: bool,
}
impl Default for FlatpakConfig {
    fn default() -> Self {
        Self { systemwide: true }
    }
}

impl Backend for Flatpak {
    type Options = FlatpakOptions;
    type Config = FlatpakConfig;

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
        config: &Self::Config,
    ) -> Result<()> {
        for (package, options) in packages {
            run_command(
                [
                    "flatpak",
                    "install",
                    if options.systemwide.unwrap_or(config.systemwide) {
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

    fn uninstall(packages: &BTreeSet<String>, no_confirm: bool, _: &Self::Config) -> Result<()> {
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

    fn update(packages: &BTreeSet<String>, no_confirm: bool, _: &Self::Config) -> Result<()> {
        run_command(
            ["flatpak", "update"]
                .into_iter()
                .chain(Some("--assumeyes").filter(|_| no_confirm))
                .chain(packages.iter().map(String::as_str)),
            Perms::Same,
        )
    }

    fn update_all(no_confirm: bool, _: &Self::Config) -> Result<()> {
        run_command(
            ["flatpak", "update"]
                .into_iter()
                .chain(Some("--assumeyes").filter(|_| no_confirm)),
            Perms::Same,
        )
    }

    fn clean_cache(config: &Self::Config) -> Result<()> {
        Self::version(config).map_or(Ok(()), |_| {
            run_command(["flatpak", "remove", "--unused"], Perms::Same)
        })
    }

    fn version(_: &Self::Config) -> Result<String> {
        run_command_for_stdout(["flatpak", "--version"], Perms::Same, false)
    }
}
