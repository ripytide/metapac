use color_eyre::Result;
use color_eyre::eyre::eyre;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Apt;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct AptConfig {}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AptPackageOptions {}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AptRepoOptions {}

impl Backend for Apt {
    type Config = AptConfig;
    type PackageOptions = AptPackageOptions;
    type RepoOptions = AptRepoOptions;

    fn invalid_package_help_text() -> String {
        indoc::formatdoc! {"
            An apt package may be invalid due to one of the following issues:
                - the package name doesn't meet the packaging requirements for a valid package name: <https://www.debian.org/doc/debian-policy/ch-controlfields.html#source>
        "}
    }

    fn is_valid_package_name(package: &str) -> Option<bool> {
        // see <https://www.debian.org/doc/debian-policy/ch-controlfields.html#source>
        let regex = Regex::new("[a-z0-9+-.]+").unwrap();

        Some(
            regex.is_match(package)
                && (package.len() >= 2)
                && package.chars().next().unwrap().is_alphanumeric(),
        )
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

        // See https://askubuntu.com/questions/2389/how-to-list-manually-installed-packages
        // for a run-down of methods for finding lists of
        // explicit/dependency packages. It doesn't seem as if apt was
        // designed with this use-case in mind so there are lots and
        // lots of different methods all of which seem to have
        // caveats.
        let explicit = run_command_for_stdout(["apt-mark", "showmanual"], Perms::Same, StdErr::Show)?;
        Ok(explicit
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
                ["apt-get", "install"]
                    .into_iter()
                    .chain(Some("--yes").filter(|_| no_confirm))
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
            run_command(
                ["apt-get", "remove"]
                    .into_iter()
                    .chain(Some("--yes").filter(|_| no_confirm))
                    .chain(packages.iter().map(String::as_str)),
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
        if !packages.is_empty() {
            run_command(
                ["apt-get", "install", "--only-upgrade"]
                    .into_iter()
                    .chain(Some("--yes").filter(|_| no_confirm))
                    .chain(packages.iter().map(String::as_str)),
                Perms::Sudo,
            )?;
        }

        Ok(())
    }

    fn update_all_packages(no_confirm: bool, _: &Self::Config) -> Result<()> {
        run_command(
            ["apt-get", "upgrade"]
                .into_iter()
                .chain(Some("--yes").filter(|_| no_confirm)),
            Perms::Sudo,
        )
    }

    fn clean_cache(config: &Self::Config) -> Result<()> {
        Self::version(config).map_or(Ok(()), |_| {
            run_command(["apt-get", "autoclean"], Perms::Sudo)
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
        run_command_for_stdout(["apt", "--version"], Perms::Same, StdErr::Show)
    }
}
