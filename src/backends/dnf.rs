use std::collections::{BTreeMap, BTreeSet};

use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Dnf;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct DnfConfig {}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DnfPackageOptions {}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DnfRepoOptions {}

impl Backend for Dnf {
    type Config = DnfConfig;
    type PackageOptions = DnfPackageOptions;
    type RepoOptions = DnfRepoOptions;

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

        let packages = run_command_for_stdout(
            [
                "dnf",
                "repoquery",
                "--userinstalled",
                "--queryformat",
                "%{name}\n",
            ],
            Perms::Same,
            false,
        )?;

        Ok(packages
            .lines()
            .map(|x| (x.to_string(), Self::PackageOptions {}))
            .collect())
    }

    fn install_packages(
        packages: &BTreeMap<String, Self::PackageOptions>,
        no_confirm: bool,
        _: &Self::Config,
    ) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["dnf", "install"]
                    .into_iter()
                    .chain(Some("--assumeyes").filter(|_| no_confirm))
                    .chain(packages.keys().map(String::as_str)),
                Perms::Sudo,
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
            // we mark as dependency and then autoremove as otherwise removing a package also
            // removes any of it's reverse dependencies which can inadvertently break group files
            // and lead to cyclic cleans/syncs
            run_command(
                ["dnf", "mark", "dependency"]
                    .into_iter()
                    .chain(Some("--assumeyes").filter(|_| no_confirm))
                    .chain(packages.iter().map(String::as_str)),
                Perms::Sudo,
            )?;

            run_command(
                ["dnf", "autoremove"]
                    .into_iter()
                    .chain(Some("--assumeyes").filter(|_| no_confirm)),
                Perms::Sudo,
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
            ["dnf", "upgrade"]
                .into_iter()
                .chain(Some("--assumeyes").filter(|_| no_confirm))
                .chain(packages.iter().map(String::as_str)),
            Perms::Sudo,
        )
    }

    fn update_all_packages(no_confirm: bool, _: &Self::Config) -> Result<()> {
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

    fn get_installed_repos(_: &Self::Config) -> Result<BTreeMap<String, Self::RepoOptions>> {
        let repos = run_command_for_stdout(["dnf", "copr", "list"], Perms::Sudo, false)?;

        let repos = repos
            .lines()
            .map(|repo| (repo.to_string(), DnfRepoOptions {}))
            .collect();

        Ok(repos)
    }

    fn add_repos(
        repos: &BTreeMap<String, Self::RepoOptions>,
        no_confirm: bool,
        _: &Self::Config,
    ) -> Result<()> {
        for repo in repos.keys() {
            run_command(
                ["dnf", "copr", "enable", repo.as_str()]
                    .into_iter()
                    .chain(Some("--assumeyes").filter(|_| no_confirm)),
                Perms::Sudo,
            )?
        }

        Ok(())
    }

    fn remove_repos(repos: &BTreeSet<String>, no_confirm: bool, _: &Self::Config) -> Result<()> {
        for repo in repos.iter() {
            run_command(
                ["dnf", "copr", "remove", repo.as_str()]
                    .into_iter()
                    .chain(Some("--assumeyes").filter(|_| no_confirm)),
                Perms::Sudo,
            )?
        }

        Ok(())
    }

    fn version(_: &Self::Config) -> Result<String> {
        run_command_for_stdout(["dnf", "--version"], Perms::Same, false)
    }
}
