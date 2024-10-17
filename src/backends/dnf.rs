use std::collections::BTreeMap;

use color_eyre::Result;
use serde::{Deserialize, Serialize};

use crate::cmd::{command_found, run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Dnf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnfQueryInfo {
    pub user: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DnfInstallOptions {
    repo: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DnfModificationOptions {}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DnfRemoveOptions {}

impl Backend for Dnf {
    type QueryInfo = DnfQueryInfo;
    type InstallOptions = DnfInstallOptions;
    type ModificationOptions = DnfModificationOptions;
    type RemoveOptions = DnfRemoveOptions;

    fn query_installed_packages(_: &Config) -> Result<BTreeMap<String, Self::QueryInfo>> {
        if !command_found("dnf") {
            return Ok(BTreeMap::new());
        }

        let system_packages = run_command_for_stdout(
            [
                "dnf",
                "repoquery",
                "--installed",
                "--queryformat",
                "%{from_repo}/%{name}",
            ],
            Perms::Same,
        )?;
        let system_packages = system_packages.lines().map(parse_package);

        let user_packages = run_command_for_stdout(
            [
                "dnf",
                "repoquery",
                "--userinstalled",
                "--queryformat",
                "%{from_repo}/%{name}",
            ],
            Perms::Same,
        )?;
        let user_packages = user_packages.lines().map(parse_package);

        Ok(system_packages
            .map(|x| (x, DnfQueryInfo { user: false }))
            .chain(user_packages.map(|x| (x, DnfQueryInfo { user: true })))
            .collect())
    }

    fn install_packages(
        packages: &BTreeMap<String, Self::InstallOptions>,
        no_confirm: bool,
        _: &Config,
    ) -> Result<()> {
        // add these two repositories as these are needed for many dependencies
        #[allow(clippy::option_if_let_else)]
        run_command(
            ["dnf", "install", "--repo", "updates", "--repo", "fedora"]
                .into_iter()
                .chain(Some("--assumeyes").filter(|_| no_confirm))
                .chain(
                    packages
                        .iter()
                        .flat_map(|(package_id, options)| match &options.repo {
                            Some(repo) => vec![package_id, "--repo", repo.as_str()],
                            None => vec![package_id.as_str()],
                        }),
                ),
            Perms::AsRoot,
        )
    }

    fn modify_packages(_: &BTreeMap<String, Self::ModificationOptions>, _: &Config) -> Result<()> {
        unimplemented!()
    }

    fn remove_packages(
        packages: &BTreeMap<String, Self::RemoveOptions>,
        no_confirm: bool,
        _: &Config,
    ) -> Result<()> {
        run_command(
            ["dnf", "remove"]
                .into_iter()
                .chain(Some("--assumeyes").filter(|_| no_confirm))
                .chain(packages.keys().map(String::as_str)),
            Perms::AsRoot,
        )
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
