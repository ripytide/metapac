use std::collections::{BTreeMap, BTreeSet};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Bun;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct BunConfig {}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BunPackageOptions {}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BunRepoOptions {}

impl Backend for Bun {
    type Config = BunConfig;
    type PackageOptions = BunPackageOptions;
    type RepoOptions = BunRepoOptions;

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

        let output =
            match run_command_for_stdout(["bun", "pm", "ls", "--global"], Perms::Same, true) {
                Ok(output) => output,
                //unfortunately when there are no global packages installed bun returns an error rather
                //than saying zero packages
                Err(_) => return Ok(BTreeMap::new()),
            };

        let lines = output.lines().collect::<Vec<_>>();

        // example output:
        // /home/ripytide/.bun/install/global node_modules (292)
        // ├── @rspack/cli@1.4.10
        // └── glob@11.0.3

        let mut packages = BTreeMap::new();
        for line in &lines[1..] {
            let tree_parts = line.split_whitespace().collect::<Vec<_>>();
            if tree_parts.len() != 2 {
                return Err(eyre!("unexpected tree parts"));
            }

            let name = if tree_parts[1].starts_with("@") {
                let package_parts = tree_parts[1].split('@').collect::<Vec<_>>();

                if package_parts.len() != 3 {
                    return Err(eyre!("unexpected package parts"));
                }

                format!("@{}", package_parts[1])
            } else {
                let package_parts = tree_parts[1].split('@').collect::<Vec<_>>();
                if package_parts.len() != 2 {
                    return Err(eyre!("unexpected package parts"));
                }

                package_parts[0].to_string()
            };

            packages.insert(name, BunPackageOptions {});
        }

        Ok(packages)
    }

    fn install_packages(
        packages: &BTreeMap<String, Self::PackageOptions>,
        _: bool,
        _: &Self::Config,
    ) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["bun", "install", "--global"]
                    .into_iter()
                    .chain(packages.keys().map(String::as_str)),
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn uninstall_packages(packages: &BTreeSet<String>, _: bool, _: &Self::Config) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["bun", "uninstall", "--global"]
                    .into_iter()
                    .chain(packages.iter().map(String::as_str)),
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn update_packages(packages: &BTreeSet<String>, _: bool, _: &Self::Config) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["bun", "update", "--global"]
                    .into_iter()
                    .chain(packages.iter().map(String::as_str)),
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn update_all_packages(_: bool, _: &Self::Config) -> Result<()> {
        run_command(["bun", "update", "--global"], Perms::Same)
    }

    fn clean_cache(config: &Self::Config) -> Result<()> {
        Self::version(config).map_or(Ok(()), |_| {
            run_command(["bun", "pm", "cache", "rm", "--global"], Perms::Same)
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
        run_command_for_stdout(["bun", "--version"], Perms::Same, false)
    }
}
