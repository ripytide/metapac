use std::collections::{BTreeMap, BTreeSet};

use color_eyre::Result;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Snap;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapOptions {
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
    fn try_from_notes(notes: &str) -> Option<SnapConfinement> {
        match notes {
            "-" => Some(SnapConfinement::Strict),
            "classic" => Some(SnapConfinement::Classic),
            "dangerous" => Some(SnapConfinement::Dangerous),
            "devmode" => Some(SnapConfinement::Devmode),
            "jailmode" => Some(SnapConfinement::Jailmode),
            _ => None,
        }
    }

    fn to_cli_option(&self) -> Option<String> {
        match self {
            SnapConfinement::Strict => None,
            SnapConfinement::Classic => Some("--classic"),
            SnapConfinement::Dangerous => Some("--dangerous"),
            SnapConfinement::Devmode => Some("--devmode"),
            SnapConfinement::Jailmode => Some("--jailmode"),
        }
        .map(String::from)
    }
}

impl Backend for Snap {
    type Options = SnapOptions;

    fn map_required(
        packages: BTreeMap<String, Package<Self::Options>>,
        _: &Config,
    ) -> Result<BTreeMap<String, Package<Self::Options>>> {
        Ok(packages)
    }

    fn query(config: &Config) -> Result<BTreeMap<String, Self::Options>> {
        if Self::version(config).is_err() {
            return Ok(BTreeMap::new());
        }

        let output = run_command_for_stdout(["snap", "list"], Perms::Same, false)?;

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
            .map(|(name, confinement)| (name.to_string(), Self::Options { confinement }))
            .collect())
    }

    fn install(packages: &BTreeMap<String, Self::Options>, _: bool, _: &Config) -> Result<()> {
        build_snap_install_commands(packages)
            .iter()
            .try_for_each(|cmd| run_command(cmd, Perms::Sudo))
    }

    fn uninstall(packages: &BTreeSet<String>, _: bool, _: &Config) -> Result<()> {
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

    fn update(packages: &BTreeSet<String>, _: bool, _: &Config) -> Result<()> {
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

    fn update_all(_: bool, _: &Config) -> Result<()> {
        run_command(["snap", "refresh"], Perms::Sudo)
    }

    fn clean_cache(config: &Config) -> Result<()> {
        Self::version(config).map_or(Ok(()), |_| {
            run_command(["rm", "-rf", "/var/lib/snapd/cache/*"], Perms::Sudo)
        })
    }

    fn version(_: &Config) -> Result<String> {
        run_command_for_stdout(["snap", "--version"], Perms::Same, false).map(|output| {
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

fn build_snap_install_commands(packages: &BTreeMap<String, SnapOptions>) -> Vec<Vec<String>> {
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
    packages_confined: Vec<(&'a String, &'a SnapOptions)>,
) -> Vec<(&'a SnapConfinement, Vec<(&'a String, &'a SnapOptions)>)> {
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
    packages_confined: Vec<(&'a String, &'a SnapOptions)>,
) -> Vec<String> {
    ["snap", "install"]
        .into_iter()
        .map(String::from)
        .chain(confinement.to_cli_option())
        .chain(packages_confined.into_iter().map(|(name, _)| name.clone()))
        .collect()
}
