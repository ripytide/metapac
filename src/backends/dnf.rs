use std::collections::{BTreeMap, BTreeSet};

use color_eyre::Result;
use serde::{Deserialize, Serialize};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Dnf;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DnfOptions {
    pub repo: Option<String>,
    pub user: Option<bool>,
}

impl Backend for Dnf {
    type Options = DnfOptions;
    type Config = ();

    fn invalid_package_help_text() -> String {
        String::new()
    }

    fn are_valid_packages(
        packages: &BTreeSet<String>,
        _: &Config,
    ) -> BTreeMap<String, Option<bool>> {
        packages.iter().map(|x| (x.to_string(), None)).collect()
    }

    fn get_installed(config: &Self::Config) -> Result<BTreeMap<String, Self::Options>> {
        if Self::version(config).is_err() {
            return Ok(BTreeMap::new());
        }

        let system_packages = run_command_for_stdout(
            [
                "dnf",
                "repoquery",
                "--installed",
                "--queryformat",
                "%{from_repo}/%{name}\n",
            ],
            Perms::Same,
            false,
        )?;
        let system_packages = system_packages.lines().map(parse_package);

        let user_packages = run_command_for_stdout(
            [
                "dnf",
                "repoquery",
                "--userinstalled",
                "--queryformat",
                "%{from_repo}/%{name}\n",
            ],
            Perms::Same,
            false,
        )?;
        let user_packages = user_packages.lines().map(parse_package);

        Ok(system_packages
            .map(|x| {
                (
                    x,
                    Self::Options {
                        user: Some(false),
                        repo: None,
                    },
                )
            })
            .chain(user_packages.map(|x| {
                (
                    x,
                    Self::Options {
                        user: Some(true),
                        repo: None,
                    },
                )
            }))
            .collect())
    }

    fn install(
        packages: &BTreeMap<String, Self::Options>,
        no_confirm: bool,
        _: &Self::Config,
    ) -> Result<()> {
        if !packages.is_empty() {
            // add these two repositories as these are needed for many dependencies
            #[allow(clippy::option_if_let_else)]
            run_command(
                ["dnf", "install", "--repo", "updates", "--repo", "fedora"]
                    .into_iter()
                    .chain(Some("--assumeyes").filter(|_| no_confirm))
                    .chain(
                        packages
                            .iter()
                            .flat_map(|(package, options)| match &options.repo {
                                Some(repo) => vec![package, "--repo", repo.as_str()],
                                None => vec![package.as_str()],
                            }),
                    ),
                Perms::Sudo,
            )?;
        }

        Ok(())
    }

    fn uninstall(packages: &BTreeSet<String>, no_confirm: bool, _: &Self::Config) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["dnf", "remove"]
                    .into_iter()
                    .chain(Some("--assumeyes").filter(|_| no_confirm))
                    .chain(packages.iter().map(String::as_str)),
                Perms::Sudo,
            )?;
        }

        Ok(())
    }

    fn update(packages: &BTreeSet<String>, no_confirm: bool, _: &Self::Config) -> Result<()> {
        run_command(
            ["dnf", "upgrade"]
                .into_iter()
                .chain(Some("--assumeyes").filter(|_| no_confirm))
                .chain(packages.iter().map(String::as_str)),
            Perms::Sudo,
        )
    }

    fn update_all(no_confirm: bool, _: &Self::Config) -> Result<()> {
        run_command(
            ["dnf", "upgrade"]
                .into_iter()
                .chain(Some("--assumeyes").filter(|_| no_confirm)),
            Perms::Sudo,
        )
    }

    fn clean_cache(config: &Self::Config) -> Result<()> {
        Self::version(config).map_or(Ok(()), |_| {
            run_command(["dnf", "clean", "all"], Perms::Same)
        })
    }

    fn version(_: &Self::Config) -> Result<String> {
        run_command_for_stdout(["dnf", "--version"], Perms::Same, false)
    }
}

fn parse_package(package: &str) -> String {
    // These repositories are ignored when storing the packages
    // as these are present by default on any sane fedora system
    if ["koji", "fedora", "updates", "anaconda", "@"]
        .iter()
        .any(|repo| package.contains(repo))
        && !package.contains("copr")
    {
        package
            .split('/')
            .nth(1)
            .expect("Cannot be empty!")
            .to_string()
    } else {
        package.to_string()
    }
}
