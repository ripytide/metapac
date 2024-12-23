use std::collections::{BTreeMap, BTreeSet};

use color_eyre::Result;
use serde::{Deserialize, Serialize};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Dnf;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DnfOptions {
    pub repo: Option<String>,
    pub user: bool,
}

impl Backend for Dnf {
    type Options = DnfOptions;

    fn map_managed_packages(
        packages: BTreeMap<String, Self::Options>,
        _: &Config,
    ) -> Result<BTreeMap<String, Self::Options>> {
        Ok(packages)
    }

    fn query(config: &Config) -> Result<BTreeMap<String, Self::Options>> {
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
                        user: false,
                        repo: None,
                    },
                )
            })
            .chain(user_packages.map(|x| {
                (
                    x,
                    Self::Options {
                        user: true,
                        repo: None,
                    },
                )
            }))
            .collect())
    }

    fn install(
        packages: &BTreeMap<String, Self::Options>,
        no_confirm: bool,
        _: &Config,
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

    fn remove(packages: &BTreeSet<String>, no_confirm: bool, _: &Config) -> Result<()> {
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

    fn clean_cache(config: &Config) -> Result<()> {
        Self::version(config).map_or(Ok(()), |_| {
            run_command(["dnf", "clean", "all"], Perms::Same)
        })
    }

    fn version(_: &Config) -> Result<String> {
        run_command_for_stdout(["dnf", "--version"], Perms::Same, false)
    }

    fn missing(managed: Self::Options, installed: Option<Self::Options>) -> Option<Self::Options> {
        match installed {
            Some(_) => None,
            None => Some(managed),
        }
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
