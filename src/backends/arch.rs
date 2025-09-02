use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use std::collections::{BTreeMap, BTreeSet};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Arch;

#[serde_inline_default]
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchOptions {}

impl Backend for Arch {
    type Options = ArchOptions;

    fn are_valid_packages(packages: BTreeSet<String>, config: &Config) -> Result<BTreeMap<String, Option<bool>>> {
        let available_packages: BTreeSet<String> = run_command_for_stdout(
            [
                config.arch.package_manager.as_command(),
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
            let validity = if available_packages.contains(&package) {
                log::warn!(
                    "{}",
                    indoc::formatdoc! {"
                        arch package {package:?} was not found as an available package and so was ignored (you can test
                        if the package exists via `pacman -Si {package:?}` or similar command using your chosen AUR helper)

                        it may be due to one of the following issues:
                            - the package name has a typo as written in your group files
                            - the package is in a repository that you don't have enabled in
                              /etc/pacman.conf (such as multilib)
                            - the package is a virtual package (https://wiki.archlinux.org/title/Pacman#Virtual_packages)
                              and so is ambiguous. You can run `pacman -Ss {package:?}` to list non-virtual packages which
                              which provide the virtual package
                            - the package was removed from the repositories
                            - the package was renamed to a different name
                            - the local package database is out of date and so doesn't yet contain the package:
                              update it with `sudo pacman -Sy` or similar command using your chosen AUR helper
                    "}
                );

                Some(false)
            } else {
                Some(true)
            };

            output.insert(package, validity);
        }

        Ok(output)
    }

    fn query(config: &Config) -> Result<BTreeMap<String, Self::Options>> {
        if Self::version(config).is_err() {
            return Ok(BTreeMap::new());
        }

        let explicit_packages = run_command_for_stdout(
            [
                config.arch.package_manager.as_command(),
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
        config: &Config,
    ) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                [
                    config.arch.package_manager.as_command(),
                    "--sync",
                    "--asexplicit",
                ]
                .into_iter()
                .chain(Some("--noconfirm").filter(|_| no_confirm))
                .chain(packages.keys().map(String::as_str)),
                config.arch.package_manager.change_perms(),
            )?;
        }

        Ok(())
    }

    fn uninstall(packages: &BTreeSet<String>, no_confirm: bool, config: &Config) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                [
                    config.arch.package_manager.as_command(),
                    "--database",
                    "--asdeps",
                ]
                .into_iter()
                .chain(packages.iter().map(String::as_str)),
                config.arch.package_manager.change_perms(),
            )?;

            let orphans_output = run_command_for_stdout(
                [
                    config.arch.package_manager.as_command(),
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
                    config.arch.package_manager.as_command(),
                    "--remove",
                    "--nosave",
                    "--recursive",
                ]
                .into_iter()
                .chain(Some("--noconfirm").filter(|_| no_confirm))
                .chain(orphans),
                config.arch.package_manager.change_perms(),
            )?;
        }

        Ok(())
    }

    fn update(packages: &BTreeSet<String>, no_confirm: bool, config: &Config) -> Result<()> {
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

    fn update_all(no_confirm: bool, config: &Config) -> Result<()> {
        run_command(
            [
                config.arch.package_manager.as_command(),
                "--sync",
                "--refresh",
                "--sysupgrade",
            ]
            .into_iter()
            .chain(Some("--noconfirm").filter(|_| no_confirm)),
            config.arch.package_manager.change_perms(),
        )
    }

    fn clean_cache(config: &Config) -> Result<()> {
        Self::version(config).map_or(Ok(()), |_| {
            run_command(
                [
                    config.arch.package_manager.as_command(),
                    "--sync",
                    "--clean",
                ],
                Perms::Same,
            )
        })
    }

    fn version(config: &Config) -> Result<String> {
        run_command_for_stdout(
            [config.arch.package_manager.as_command(), "--version"],
            Perms::Same,
            false,
        )
    }
}
