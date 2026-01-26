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
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct FlatpakConfig {}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlatpakPackageOptions {
    pub remote: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlatpakRepoOptions {
    pub url: Option<String>,
}

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

        let apps = run_command_for_stdout(
            [
                "flatpak",
                "list",
                "--app",
                "--columns=installation,application,origin",
            ],
            Perms::Same,
            StdErr::Show,
        )?;

        let apps = apps.lines().map(|line| {
            let parts = line.split_whitespace().collect::<Vec<_>>();

            (
                format!("{}:{}", parts[0], parts[1]),
                Self::PackageOptions {
                    remote: Some(parts[2].to_string()),
                },
            )
        });

        Ok(apps.collect())
    }

    fn install_packages(
        packages: &BTreeMap<String, Self::PackageOptions>,
        no_confirm: bool,
        _: &Self::Config,
    ) -> Result<()> {
        //group packages for faster installation and less y/n prompts
        let mut groups: BTreeMap<(String, Option<String>), Vec<String>> = BTreeMap::new();
        for (package, options) in packages {
            let (installation, name) = package.split_once(":").ok_or(eyre!(
                "invalid flatpak package name: {package:?}, should be in form \"installation:package\", such as \"system:metapac\""
            ))?;

            groups
                .entry((installation.to_string(), options.remote.clone()))
                .or_default()
                .push(name.to_string());
        }

        for ((installation, remote), packages) in groups {
            run_command(
                ["flatpak", "install"]
                    .into_iter()
                    .chain(Some("--assumeyes").filter(|_| no_confirm))
                    .map(|x| x.to_string())
                    .chain(match installation.as_str() {
                        "user" => Some("--user".to_string()),
                        "system" => Some("--system".to_string()),
                        x => Some(format!("--installation={x}")),
                    })
                    .chain(remote)
                    .chain(packages),
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
        //group packages for faster uninstallation and less y/n prompts
        let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for package in packages {
            let (installation, name) = package.split_once(":").ok_or(eyre!(
                "invalid flatpak package name: {package:?}, should be in form \"installation:package\", such as \"system:metapac\""
            ))?;

            groups
                .entry(installation.to_string())
                .or_default()
                .push(name.to_string());
        }

        for (installation, packages) in groups {
            run_command(
                ["flatpak", "uninstall"]
                    .into_iter()
                    .chain(Some("--assumeyes").filter(|_| no_confirm))
                    .map(|x| x.to_string())
                    .chain(match installation.as_str() {
                        "user" => Some("--user".to_string()),
                        "system" => Some("--system".to_string()),
                        x => Some(format!("--installation={x}")),
                    })
                    .chain(packages),
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
        //group packages for faster uninstallation and less y/n prompts
        let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for package in packages {
            let (installation, name) = package.split_once(":").ok_or(eyre!(
                "invalid flatpak package name: {package:?}, should be in form \"installation:package\", such as \"system:metapac\""
            ))?;

            groups
                .entry(installation.to_string())
                .or_default()
                .push(name.to_string());
        }

        for (installation, packages) in groups {
            run_command(
                ["flatpak", "update"]
                    .into_iter()
                    .chain(Some("--assumeyes").filter(|_| no_confirm))
                    .map(|x| x.to_string())
                    .chain(match installation.as_str() {
                        "user" => Some("--user".to_string()),
                        "system" => Some("--system".to_string()),
                        x => Some(format!("--installation={x}")),
                    })
                    .chain(packages),
                Perms::Same,
            )?;
        }

        Ok(())
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
        let repos = run_command_for_stdout(
            ["flatpak", "remotes", "--columns", "options,name,url"],
            Perms::Same,
            StdErr::Show,
        )?;

        let repos = repos
            .lines()
            // if there are no remotes an empty line is still returned
            // so we filter out empty lines
            .filter(|x| !x.is_empty())
            .map(|line| {
                let parts = line.split_whitespace().collect::<Vec<_>>();
                let installation = parts[0].split(",").collect::<Vec<_>>()[0];
                (
                    format!("{}:{}", installation, parts[1]),
                    FlatpakRepoOptions {
                        url: Some(parts[2].to_string()),
                    },
                )
            })
            .collect();

        Ok(repos)
    }

    fn add_repos(
        repos: &BTreeMap<String, Self::RepoOptions>,
        _: bool,
        _: &Self::Config,
    ) -> Result<()> {
        for (repo, options) in repos {
            let (installation, name) = repo.split_once(":").ok_or(eyre!(
                "invalid flatpak repo name: {repo:?}, should be in form \"installation:repo\", such as \"system:flathub\""
            ))?;

            run_command(
                ["flatpak", "remote-add"]
                    .into_iter()
                    .map(ToString::to_string)
                    .chain(match installation {
                        "user" => Some("--user".to_string()),
                        "system" => Some("--system".to_string()),
                        x => Some(format!("--installation={x}")),
                    })
                    .chain([
                        name.to_string(),
                        options
                            .url
                            .as_deref()
                            .ok_or(eyre!("flatpak repos must have the \"url\" option set"))?
                            .to_string(),
                    ]),
                Perms::Same,
            )?
        }

        Ok(())
    }

    fn remove_repos(repos: &BTreeSet<String>, _: bool, _: &Self::Config) -> Result<()> {
        for repo in repos {
            let (installation, name) = repo.split_once(":").ok_or(eyre!(
                "invalid flatpak repo name: {repo:?}, should be in form \"installation:repo\", such as \"system:flathub\""
            ))?;

            run_command(
                ["flatpak", "remote-delete"]
                    .into_iter()
                    .map(ToString::to_string)
                    .chain(match installation {
                        "user" => Some("--user".to_string()),
                        "system" => Some("--system".to_string()),
                        x => Some(format!("--installation={x}")),
                    })
                    .chain([name.to_string()]),
                Perms::Same,
            )?
        }

        Ok(())
    }

    fn version(_: &Self::Config) -> Result<String> {
        run_command_for_stdout(["flatpak", "--version"], Perms::Same, StdErr::Show)
    }
}
