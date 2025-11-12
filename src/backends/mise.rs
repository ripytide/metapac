use std::collections::{BTreeMap, BTreeSet};

use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Mise;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MiseOptions {
    #[serde(default)]
    version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct MiseConfig {}

impl Backend for Mise {
    type Options = MiseOptions;
    type Config = MiseConfig;

    fn invalid_package_help_text() -> String {
        String::new()
    }

    fn is_valid_package_name(_: &str) -> Option<bool> {
        None
    }

    fn get_all(_: &Self::Config) -> Result<BTreeSet<String>> {
        let search = run_command_for_stdout(
            ["mise", "search", "--no-headers", "--quiet"],
            Perms::Same,
            true,
        )?;

        Ok(search
            .lines()
            .map(|line| {
                line.split_whitespace()
                    .next()
                    .expect("mise search lines should not be empty")
                    .to_string()
            })
            .collect())
    }

    fn get_installed(config: &Self::Config) -> Result<BTreeMap<String, Self::Options>> {
        if Self::version(config).is_err() {
            return Ok(BTreeMap::new());
        }

        let packages = run_command_for_stdout(
            ["mise", "ls", "--installed", "--json", "--quiet"],
            Perms::Same,
            true,
        )?;

        let packages_json = match serde_json::from_str(&packages)? {
            Value::Object(x) => x,
            _ => return Err(eyre!("json should be an object")),
        };

        let mut packages = BTreeMap::new();
        for (key, value) in packages_json {
            // Each package maps to an array of version objects
            let versions = value
                .as_array()
                .ok_or(eyre!("mise package {key:?} should be an array"))?;

            // Take the first version and ignore any others
            if let Some(first_version) = versions.first() {
                packages.insert(
                    key.clone(),
                    MiseOptions {
                        version: first_version
                            .get("version")
                            .and_then(|x| x.as_str())
                            .map(|x| x.to_string()),
                    },
                );
            }
        }

        Ok(packages)
    }

    fn install(
        packages: &BTreeMap<String, Self::Options>,
        no_confirm: bool,
        _: &Self::Config,
    ) -> Result<()> {
        for (package, options) in packages {
            let package = format!("{package}@{}", options.version.as_deref().unwrap_or(""));
            run_command(
                ["mise", "install"]
                    .into_iter()
                    .chain(Some("--yes").filter(|_| no_confirm))
                    .chain(std::iter::once(package.as_str())),
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn uninstall(packages: &BTreeSet<String>, _: bool, _: &Self::Config) -> Result<()> {
        for package in packages {
            run_command(["mise", "uninstall", package], Perms::Same)?;
        }

        Ok(())
    }

    fn update(packages: &BTreeSet<String>, _: bool, _: &Self::Config) -> Result<()> {
        for package in packages {
            run_command(["mise", "upgrade", package], Perms::Same)?;
        }

        Ok(())
    }

    fn update_all(_: bool, _: &Self::Config) -> Result<()> {
        run_command(["mise", "upgrade"], Perms::Same)
    }

    fn clean_cache(_: &Self::Config) -> Result<()> {
        Ok(())
    }

    fn version(_: &Self::Config) -> Result<String> {
        run_command_for_stdout(["mise", "--version"], Perms::Same, false)
    }
}
