use color_eyre::Result;
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use std::collections::{BTreeMap, BTreeSet};

use crate::cmd::{command_found, run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Arch;

#[derive(Debug, Clone)]
pub struct ArchQueryInfo {}

#[serde_inline_default]
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ArchInstallOptions {
    #[serde_inline_default(ArchInstallOptions::default().optional_deps)]
    pub optional_deps: Vec<String>,
}

impl Backend for Arch {
    type QueryInfo = ArchQueryInfo;
    type InstallOptions = ArchInstallOptions;

    fn include_implicit(key: &str, value: &Self::InstallOptions) -> Vec<String> {
        let mut packages = vec![key.to_owned()];

        for package in &value.optional_deps {
            packages.push(package.clone());
        }

        packages
    }

    fn map_managed_packages(
        mut packages: BTreeMap<String, Self::InstallOptions>,
        config: &Config,
    ) -> Result<BTreeMap<String, Self::InstallOptions>> {
        if !command_found(config.arch_package_manager.as_command()) {
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
        )?;

        for group in groups.lines() {
            if let Some(install_options) = packages.remove(group) {
                let group_packages = run_command_for_stdout(
                    [
                        config.arch_package_manager.as_command(),
                        "--sync",
                        "--groups",
                        "--quiet",
                        group,
                    ],
                    Perms::Same,
                )?;

                for group_package in group_packages.lines() {
                    let mut overridden = packages
                        .insert(group_package.to_string(), Self::InstallOptions::default())
                        .is_some();
                    for opts in &install_options.optional_deps {
                        overridden |= packages
                            .insert(opts.to_string(), Self::InstallOptions::default())
                            .is_some();
                    }

                    if overridden {
                        log::warn!("arch package {group_package} or one of its dependencies has been overridden by the {group} package group");
                    }
                }
            }
        }

        Ok(packages)
    }

    fn query_installed_packages(config: &Config) -> Result<BTreeMap<String, Self::QueryInfo>> {
        if !command_found(config.arch_package_manager.as_command()) {
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
        )?;

        let mut result = BTreeMap::new();

        for package_id in explicit_packages.lines() {
            result.insert(package_id.to_string(), ArchQueryInfo {});
        }

        Ok(result)
    }

    fn install_packages(
        packages: &BTreeMap<String, Self::InstallOptions>,
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
                .chain(packages.keys().map(String::as_str))
                .chain(
                    packages.values().flat_map(|dependencies| {
                        dependencies.optional_deps.iter().map(String::as_str)
                    }),
                ),
                config.arch_package_manager.change_perms(),
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
}
