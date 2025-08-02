use color_eyre::Result;
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

    fn map_required(
        mut packages: BTreeMap<String, Package<Self::Options>>,
        config: &Config,
    ) -> Result<BTreeMap<String, Package<Self::Options>>> {
        if Self::version(config).is_err() {
            return Ok(BTreeMap::new());
        }

        let groups = run_command_for_stdout(
            [
                config.arch_package_manager.as_command(),
                "--sync",
                "--groups",
                "--quiet",
            ],
            Perms::Same,
            false,
        )?;

        for group in groups.lines() {
            if let Some(options) = packages.remove(group) {
                let group_packages = run_command_for_stdout(
                    [
                        config.arch_package_manager.as_command(),
                        "--sync",
                        "--groups",
                        "--quiet",
                        group,
                    ],
                    Perms::Same,
                    false,
                )?;

                for group_package in group_packages.lines() {
                    let overridden = packages
                        .insert(group_package.to_string(), options.clone())
                        .is_some();

                    if overridden {
                        log::warn!(
                            "arch package {group_package:?} has been overridden by the {group:?} package group"
                        );
                    }
                }
            }
        }

        let all_packages: BTreeSet<String> = run_command_for_stdout(
            [
                config.arch_package_manager.as_command(),
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

        let packages_cloned = packages.keys().cloned().collect::<Vec<_>>();
        for package in packages_cloned {
            let is_real_package = all_packages.contains(&package);

            if !is_real_package {
                packages.remove(&package);

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
            }
        }

        Ok(packages)
    }

    fn query(config: &Config) -> Result<BTreeMap<String, Self::Options>> {
        if Self::version(config).is_err() {
            return Ok(BTreeMap::new());
        }

        let explicit_packages = run_command_for_stdout(
            [
                config.arch_package_manager.as_command(),
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
                    config.arch_package_manager.as_command(),
                    "--sync",
                    "--asexplicit",
                ]
                .into_iter()
                .chain(Some("--no_confirm").filter(|_| no_confirm))
                .chain(packages.keys().map(String::as_str)),
                config.arch_package_manager.change_perms(),
            )?;
        }

        Ok(())
    }

    fn uninstall(packages: &BTreeSet<String>, no_confirm: bool, config: &Config) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                [
                    config.arch_package_manager.as_command(),
                    "--database",
                    "--asdeps",
                ]
                .into_iter()
                .chain(packages.iter().map(String::as_str)),
                config.arch_package_manager.change_perms(),
            )?;

            let orphans_output = run_command_for_stdout(
                [
                    config.arch_package_manager.as_command(),
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
                    config.arch_package_manager.as_command(),
                    "--remove",
                    "--nosave",
                    "--recursive",
                ]
                .into_iter()
                .chain(Some("--noconfirm").filter(|_| no_confirm))
                .chain(orphans),
                config.arch_package_manager.change_perms(),
            )?;
        }

        Ok(())
    }

    fn clean_cache(config: &Config) -> Result<()> {
        Self::version(config).map_or(Ok(()), |_| {
            run_command(
                [
                    config.arch_package_manager.as_command(),
                    "--sync",
                    "--clean",
                ],
                Perms::Same,
            )
        })
    }

    fn version(config: &Config) -> Result<String> {
        run_command_for_stdout(
            [config.arch_package_manager.as_command(), "--version"],
            Perms::Same,
            false,
        )
    }

    fn update(config: &Config) -> Result<()> {
        run_command(
            [
                config.arch_package_manager.as_command(),
                "--sync",
                "--refresh",
                "--sysupgrade",
            ],
            config.arch_package_manager.change_perms(),
        )?;
        Ok(())
    }
}
