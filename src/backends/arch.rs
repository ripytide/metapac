use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Arch;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchOptions {}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ArchConfig {
    #[serde(default)]
    pub package_manager: ArchPackageManager,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchPackageManager {
    #[default]
    Pacman,
    Pamac,
    Paru,
    Pikaur,
    Yay,
}
impl ArchPackageManager {
    pub fn as_command(&self) -> &'static str {
        match self {
            ArchPackageManager::Pacman => "pacman",
            ArchPackageManager::Pamac => "pamac",
            ArchPackageManager::Paru => "paru",
            ArchPackageManager::Pikaur => "pikaur",
            ArchPackageManager::Yay => "yay",
        }
    }

    pub fn change_perms(&self) -> Perms {
        match self {
            ArchPackageManager::Pacman => Perms::Sudo,
            ArchPackageManager::Pamac => Perms::Same,
            ArchPackageManager::Paru => Perms::Same,
            ArchPackageManager::Pikaur => Perms::Same,
            ArchPackageManager::Yay => Perms::Same,
        }
    }
}

impl Backend for Arch {
    type Options = ArchOptions;
    type Config = ArchConfig;

    fn invalid_package_help_text() -> String {
        indoc::formatdoc! {"
            A package may be invalid due to one of the following issues:
                - the package name has a typo as written in your group files
                - the package is in a repository that you don't have enabled in
                    /etc/pacman.conf (such as multilib)
                - the package is a virtual package (https://wiki.archlinux.org/title/Pacman#Virtual_packages)
                    and so is ambiguous. You can run `pacman -Ss <virtual_package>` to list non-virtual packages which
                    which provide the virtual package
                - the package was removed from the repositories
                - the package was renamed to a different name
                - the local package database is out of date and so doesn't yet contain the package:
                    update it with `sudo pacman -Sy` or similar command using your chosen AUR helper
                - the package is actually a package group which is not valid in metapac group files, see <https://github.com/ripytide/metapac#arch>

            You can check to see if the package exists via `pacman -Si <package>` or a similar command using your chosen AUR helper.
        "}
    }

    fn are_valid_packages(
        packages: &BTreeSet<String>,
        config: &Config,
    ) -> BTreeMap<String, Option<bool>> {
        match inner_are_valid_packages(packages, config) {
            Ok(x) => x,
            Err(_) => packages.iter().map(|x| (x.to_string(), None)).collect(),
        }
    }

    fn query(config: &Self::Config) -> Result<BTreeMap<String, Self::Options>> {
        if Self::version(config).is_err() {
            return Ok(BTreeMap::new());
        }

        let explicit_packages = run_command_for_stdout(
            [
                config.package_manager.as_command(),
                "--query",
                "--explicit",
                "--quiet",
            ],
            Perms::Same,
            false,
        )?;

        let mut result = BTreeMap::new();

        for package in explicit_packages.lines() {
            result.insert(package.to_string(), Self::Options {});
        }

        Ok(result)
    }

    fn install(
        packages: &BTreeMap<String, Self::Options>,
        no_confirm: bool,
        config: &Self::Config,
    ) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                [
                    config.package_manager.as_command(),
                    "--sync",
                    "--asexplicit",
                ]
                .into_iter()
                .chain(Some("--noconfirm").filter(|_| no_confirm))
                .chain(packages.keys().map(String::as_str)),
                config.package_manager.change_perms(),
            )?;
        }

        Ok(())
    }

    fn uninstall(
        packages: &BTreeSet<String>,
        no_confirm: bool,
        config: &Self::Config,
    ) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                [
                    config.package_manager.as_command(),
                    "--database",
                    "--asdeps",
                ]
                .into_iter()
                .chain(packages.iter().map(String::as_str)),
                config.package_manager.change_perms(),
            )?;

            let orphans_output = run_command_for_stdout(
                [
                    config.package_manager.as_command(),
                    "--query",
                    "--deps",
                    "--unrequired",
                    "--quiet",
                ],
                Perms::Same,
                false,
            )?;
            let orphans = orphans_output.lines();

            run_command(
                [
                    config.package_manager.as_command(),
                    "--remove",
                    "--nosave",
                    "--recursive",
                ]
                .into_iter()
                .chain(Some("--noconfirm").filter(|_| no_confirm))
                .chain(orphans),
                config.package_manager.change_perms(),
            )?;
        }

        Ok(())
    }

    fn update(packages: &BTreeSet<String>, no_confirm: bool, config: &Self::Config) -> Result<()> {
        let installed = Self::query(config)?;
        let installed_names = installed.keys().map(String::from).collect();

        let difference = packages
            .difference(&installed_names)
            .collect::<BTreeSet<_>>();

        if !difference.is_empty() {
            return Err(eyre!("{difference:?} packages are not installed"));
        }

        let install_options = installed
            .clone()
            .into_iter()
            .filter(|(x, _)| packages.contains(x))
            .collect();

        Self::install(&install_options, no_confirm, config)
    }

    fn update_all(no_confirm: bool, config: &Self::Config) -> Result<()> {
        run_command(
            [
                config.package_manager.as_command(),
                "--sync",
                "--refresh",
                "--sysupgrade",
            ]
            .into_iter()
            .chain(Some("--noconfirm").filter(|_| no_confirm)),
            config.package_manager.change_perms(),
        )
    }

    fn clean_cache(config: &Self::Config) -> Result<()> {
        Self::version(config).map_or(Ok(()), |_| {
            run_command(
                [config.package_manager.as_command(), "--sync", "--clean"],
                Perms::Same,
            )
        })
    }

    fn version(config: &Self::Config) -> Result<String> {
        run_command_for_stdout(
            [config.package_manager.as_command(), "--version"],
            Perms::Same,
            false,
        )
    }
}

fn inner_are_valid_packages(
    packages: &BTreeSet<String>,
    config: &Config,
) -> Result<BTreeMap<String, Option<bool>>> {
    let available_packages: BTreeSet<String> = run_command_for_stdout(
        [
            config.backends.arch.package_manager.as_command(),
            "--sync",
            "--list",
            "--quiet",
        ],
        Perms::Same,
        false,
    )?
    .lines()
    .map(String::from)
    .collect();

    let mut output = BTreeMap::new();
    for package in packages {
        output.insert(
            package.to_string(),
            Some(available_packages.contains(package)),
        );
    }

    Ok(output)
}
