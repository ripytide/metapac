use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;

use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct WinGet;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct WinGetConfig {}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WinGetPackageOptions {}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WinGetRepoOptions {}

impl Backend for WinGet {
    type Config = WinGetConfig;
    type PackageOptions = WinGetPackageOptions;
    type RepoOptions = WinGetRepoOptions;

    fn invalid_package_help_text() -> String {
        indoc::formatdoc! {"
            A winget package may be invalid due to one of the following issues:
                - the package name does not use the explicit \"publisher.package\" format which is
                  required by metapac in order to unambiguously match installed packages with those
                  declared in your group files
        "}
    }

    fn is_valid_package_name(package: &str) -> Option<bool> {
        // metapac requires the explicit form of a package which is `publisher.package`
        if package.chars().filter(|x| *x == '.').count() == 0 {
            Some(false)
        } else {
            None
        }
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

        //TODO: refactor if https://github.com/microsoft/winget-cli/issues/184 or https://github.com/microsoft/winget-cli/issues/4267 are ever fixed
        let mut tempfile = tempfile::NamedTempFile::new()?;
        let _ = run_command_for_stdout(
            [
                "winget",
                "export",
                "--nowarn",
                tempfile.path().to_str().unwrap(),
            ],
            Perms::Same,
            false,
        )?;

        let mut export = String::new();
        tempfile.read_to_string(&mut export)?;

        let export: Value = serde_json::from_str(&export)?;

        Ok(export["Sources"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|x| x["Packages"].as_array().unwrap())
            .map(|x| {
                (
                    x["PackageIdentifier"].as_str().unwrap().to_string(),
                    Self::PackageOptions {},
                )
            })
            .collect())
    }

    fn install_packages(
        packages: &BTreeMap<String, Self::PackageOptions>,
        _: bool,
        _: &Self::Config,
    ) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["winget", "install"]
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
                ["winget", "uninstall"]
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
                ["winget", "update"]
                    .into_iter()
                    .chain(packages.iter().map(String::as_str)),
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn update_all_packages(_: bool, _: &Self::Config) -> Result<()> {
        run_command(["winget", "update", "--recurse"], Perms::Same)
    }

    // currently there is no way to do it for winget, see
    // https://github.com/microsoft/winget-cli/issues/343
    fn clean_cache(_: &Self::Config) -> Result<()> {
        Ok(())
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
        run_command_for_stdout(["winget", "--version"], Perms::Same, false)
    }
}
