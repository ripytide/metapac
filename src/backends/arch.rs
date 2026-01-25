use color_eyre::Result;
use color_eyre::eyre::eyre;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Arch;

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

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchPackageOptions {}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchRepoOptions {}

impl Backend for Arch {
    type Config = ArchConfig;
    type PackageOptions = ArchPackageOptions;
    type RepoOptions = ArchRepoOptions;

    fn invalid_package_help_text() -> String {
        indoc::formatdoc! {"
            An arch package may be invalid due to one of the following issues:
                - the package name doesn't meet the packaging requirements for a valid package name: <https://wiki.archlinux.org/title/Arch_package_guidelines#Package_naming>
                - the package is in a repository that you don't have enabled in
                  /etc/pacman.conf (such as multilib)
                - the package is a virtual package (https://wiki.archlinux.org/title/Pacman#Virtual_packages)
                  and so is ambiguous. You can run `pacman -Ss <virtual_package>` to list non-virtual packages which
                  which provide the virtual package
                - the package was removed from the repositories
                - the package was renamed to a different name
                - the local package database is out of date and so doesn't yet contain the package,
                  update it with `sudo pacman -Sy` or similar command using your chosen AUR helper
                - the package is actually a package group which is not valid in metapac group files,
                  see <https://github.com/ripytide/metapac#arch>

            You can check to see if the package exists via `pacman -Si <package>` or a similar command using your chosen AUR helper.
        "}
    }

    fn is_valid_package_name(package: &str) -> Option<bool> {
        // see <https://wiki.archlinux.org/title/Arch_package_guidelines#Package_naming>
        let regex = Regex::new("[a-z0-9@._+-]+").unwrap();

        Some(regex.is_match(package) && !package.starts_with("-") && !package.starts_with("."))
    }

    fn get_all_packages(config: &Self::Config) -> Result<BTreeSet<String>> {
        let all = run_command_for_stdout(
            [
                config.package_manager.as_command(),
                "--sync",
                "--list",
                "--quiet",
            ],
            Perms::Same,
            StdErr::Show,
        )?;

        let installed = run_command_for_stdout(
            [config.package_manager.as_command(), "--query", "--quiet"],
            Perms::Same,
            StdErr::Show,
        )?;

        Ok(all
            .lines()
            .chain(installed.lines())
            .map(String::from)
            .collect())
    }

    fn get_installed_packages(
        config: &Self::Config,
    ) -> Result<BTreeMap<String, Self::PackageOptions>> {
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
            StdErr::Show,
        )?;

        let mut result = BTreeMap::new();

        for package in explicit_packages.lines() {
            result.insert(package.to_string(), Self::PackageOptions {});
        }

        Ok(result)
    }

    fn install_packages(
        packages: &BTreeMap<String, Self::PackageOptions>,
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

    fn uninstall_packages(
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
                StdErr::Show,
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

    fn update_packages(
        packages: &BTreeSet<String>,
        no_confirm: bool,
        config: &Self::Config,
    ) -> Result<()> {
        let installed = Self::get_installed_packages(config)?;
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

        Self::install_packages(&install_options, no_confirm, config)
    }

    fn update_all_packages(no_confirm: bool, config: &Self::Config) -> Result<()> {
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

    fn get_installed_repos(_: &Self::Config) -> Result<BTreeMap<String, Self::RepoOptions>> {
        Ok(BTreeMap::new())
    }

    fn add_repos(_: &BTreeMap<String, Self::RepoOptions>, _: bool, _: &Self::Config) -> Result<()> {
        Err(eyre!("unimplemented"))
    }

    fn remove_repos(_: &BTreeSet<String>, _: bool, _: &Self::Config) -> Result<()> {
        Err(eyre!("unimplemented"))
    }

    fn version(config: &Self::Config) -> Result<String> {
        run_command_for_stdout(
            [config.package_manager.as_command(), "--version"],
            Perms::Same,
            StdErr::Show,
        )
    }
}
