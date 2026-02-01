use std::collections::{BTreeMap, BTreeSet};

use color_eyre::Result;
use color_eyre::eyre::eyre;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Snap;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct SnapConfig {}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapPackageOptions {
    pub confinement: Option<SnapConfinement>,
}

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, derive_more::Display, Serialize, Deserialize, Hash,
)]
#[serde(rename_all = "snake_case")]
pub enum SnapConfinement {
    Strict,
    Classic,
    Dangerous,
    Devmode,
    Jailmode,
}
impl SnapConfinement {
    fn try_from_notes(notes: &str) -> Option<Self> {
        match notes {
            "-" => Some(Self::Strict),
            "classic" => Some(Self::Classic),
            "dangerous" => Some(Self::Dangerous),
            "devmode" => Some(Self::Devmode),
            "jailmode" => Some(Self::Jailmode),
            _ => None,
        }
    }

    fn to_cli_option(&self) -> Option<String> {
        match self {
            Self::Strict => None,
            Self::Classic => Some("--classic"),
            Self::Dangerous => Some("--dangerous"),
            Self::Devmode => Some("--devmode"),
            Self::Jailmode => Some("--jailmode"),
        }
        .map(String::from)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapRepoOptions {}

impl Backend for Snap {
    type Config = SnapConfig;
    type PackageOptions = SnapPackageOptions;
    type RepoOptions = SnapRepoOptions;

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

        let output = run_command_for_stdout(["snap", "list"], Perms::Same, StdErr::Show)?;

        // Skip the first line which is the header
        Ok(output
            .lines()
            .skip(1)
            .filter_map(|line| {
                let mut fields = line.split_whitespace();
                fields.next().map(|name|
                    // skip "Version", "Rev", "Tracking", and "Publisher" fields
                    (name, fields.nth(4).and_then(SnapConfinement::try_from_notes)))
            })
            .map(|(name, confinement)| (name.to_string(), Self::PackageOptions { confinement }))
            .collect())
    }

    fn install_packages(
        packages: &BTreeMap<String, Self::PackageOptions>,
        _: bool,
        _: &Self::Config,
    ) -> Result<()> {
        build_snap_install_commands(packages)
            .iter()
            .try_for_each(|cmd| run_command(cmd, Perms::Sudo))
    }

    fn uninstall_packages(packages: &BTreeSet<String>, _: bool, _: &Self::Config) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["snap", "remove"]
                    .into_iter()
                    .chain(packages.iter().map(String::as_str)),
                Perms::Sudo,
            )?;
        }

        Ok(())
    }

    fn update_packages(packages: &BTreeSet<String>, _: bool, _: &Self::Config) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["snap", "refresh"]
                    .into_iter()
                    .chain(packages.iter().map(String::as_str)),
                Perms::Sudo,
            )?;
        }

        Ok(())
    }

    fn update_all_packages(_: bool, _: &Self::Config) -> Result<()> {
        run_command(["snap", "refresh"], Perms::Sudo)
    }

    fn clean_cache(config: &Self::Config) -> Result<()> {
        Self::version(config).map_or(Ok(()), |_| {
            run_command(["rm", "-rf", "/var/lib/snapd/cache/*"], Perms::Sudo)
        })
    }

    fn get_installed_repos(_: &Self::Config) -> Result<BTreeMap<String, Self::RepoOptions>> {
        Ok(BTreeMap::new())
    }

    fn add_repos(
        repos: &BTreeMap<String, Self::RepoOptions>,
        _: bool,
        _: &Self::Config,
    ) -> Result<()> {
        if repos.is_empty() {
            Ok(())
        } else {
            Err(eyre!("unimplemented"))
        }
    }

    fn remove_repos(repos: &BTreeSet<String>, _: bool, _: &Self::Config) -> Result<()> {
        if repos.is_empty() {
            Ok(())
        } else {
            Err(eyre!("unimplemented"))
        }
    }

    fn version(_: &Self::Config) -> Result<String> {
        run_command_for_stdout(["snap", "--version"], Perms::Same, StdErr::Show).map(|output| {
            output
                .lines()
                .next()
                .unwrap_or_default()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        })
    }
}

fn build_snap_install_commands(
    packages: &BTreeMap<String, SnapPackageOptions>,
) -> Vec<Vec<String>> {
    packages
        .iter()
        .map(|(name, options)| {
            (
                options
                    .confinement
                    .as_ref()
                    .unwrap_or(&SnapConfinement::Strict),
                (name, options),
            )
        })
        .into_group_map()
        .into_iter()
        .sorted()
        .map(|(confinement, packages_confined)| atomize_not_strict(confinement, packages_confined))
        .flat_map(Vec::into_iter)
        .map(|(confinement, packages_confined)| {
            build_snap_install_command(confinement, packages_confined)
        })
        .collect()
}

fn atomize_not_strict<'a>(
    confinement: &'a SnapConfinement,
    packages_confined: Vec<(&'a String, &'a SnapPackageOptions)>,
) -> Vec<(
    &'a SnapConfinement,
    Vec<(&'a String, &'a SnapPackageOptions)>,
)> {
    match confinement.to_cli_option() {
        Some(_) => packages_confined
            .into_iter()
            .map(|name2options| (confinement, vec![name2options]))
            .collect(),
        None => vec![(confinement, packages_confined)],
    }
}

fn build_snap_install_command<'a>(
    confinement: &'a SnapConfinement,
    packages_confined: Vec<(&'a String, &'a SnapPackageOptions)>,
) -> Vec<String> {
    ["snap", "install"]
        .into_iter()
        .map(String::from)
        .chain(confinement.to_cli_option())
        .chain(packages_confined.into_iter().map(|(name, _)| name.clone()))
        .collect()
}
