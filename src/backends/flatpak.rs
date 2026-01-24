use std::collections::{BTreeMap, BTreeSet};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Flatpak;

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

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlatpakPackageOptions {
    pub systemwide: Option<bool>,
    pub remote: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlatpakRepoOptions {}

impl Backend for Flatpak {
    type Config = FlatpakConfig;
    type PackageOptions = FlatpakPackageOptions;
    type RepoOptions = FlatpakRepoOptions;

    fn invalid_package_help_text() -> String {
        String::new()
    }

    fn is_valid_package_name(_: &str) -> Option<bool> {
        None
    }

    fn get_all_packages(_: &Self::Config) -> Result<BTreeSet<String>> {
        Err(eyre!("unimplemented"))
    }

    fn get_installed_packages(
        config: &Self::Config,
    ) -> Result<BTreeMap<String, Self::PackageOptions>> {
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
                Self::PackageOptions {
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
                Self::PackageOptions {
                    systemwide: Some(false),
                    remote: None,
                },
            )
        });

        Ok(system_apps.chain(user_apps).collect())
    }

    fn install_packages(
        packages: &BTreeMap<String, Self::PackageOptions>,
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

    fn uninstall_packages(
        packages: &BTreeSet<String>,
        no_confirm: bool,
        _: &Self::Config,
    ) -> Result<()> {
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

    fn update_packages(
        packages: &BTreeSet<String>,
        no_confirm: bool,
        _: &Self::Config,
    ) -> Result<()> {
        run_command(
            ["flatpak", "update"]
                .into_iter()
                .chain(Some("--assumeyes").filter(|_| no_confirm))
                .chain(packages.iter().map(String::as_str)),
            Perms::Same,
        )
    }

    fn update_all_packages(no_confirm: bool, _: &Self::Config) -> Result<()> {
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

    fn get_installed_repos(_: &Self::Config) -> Result<BTreeMap<String, Self::RepoOptions>> {
        Ok(BTreeMap::new())
    }

    fn add_repos(_: &BTreeMap<String, Self::RepoOptions>, _: bool, _: &Self::Config) -> Result<()> {
        Err(eyre!("unimplemented"))
    }

    fn remove_repos(_: &BTreeSet<String>, _: bool, _: &Self::Config) -> Result<()> {
        Err(eyre!("unimplemented"))
    }

    fn version(_: &Self::Config) -> Result<String> {
        run_command_for_stdout(["flatpak", "--version"], Perms::Same, false)
    }
}
