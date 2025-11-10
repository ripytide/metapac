use std::collections::{BTreeMap, BTreeSet};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Mise;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(default)]
pub struct MiseOptions {
    requested_version: Option<String>,
    active: Option<bool>,
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
        Err(eyre!("unimplemented"))
    }

    fn get_installed(config: &Self::Config) -> Result<BTreeMap<String, Self::Options>> {
        if Self::version(config).is_err() {
            return Ok(BTreeMap::new());
        }

        let stdout = run_command_for_stdout(
            ["mise", "ls", "--global", "--json", "--installed"],
            Perms::Same,
            true,
        )?;

        let value: Value = serde_json::from_str(&stdout)?;
        let mise_conf = match value {
            Value::Object(x) => x,
            _ => return Err(eyre!("json should be an object,")),
        };

        let mut packages = BTreeMap::new();
        for (key, value) in mise_conf {
            // Each package maps to an array of version objects
            let versions = value
                .as_array()
                .ok_or(eyre!("mise package {key:?} should be an array"))?;

            // Take the first version (or we could handle multiple versions)
            if let Some(first_version) = versions.first() {
                packages.insert(
                    key.clone(),
                    serde_json::from_value(first_version.clone()).map_err(|err| {
                        eyre!("mise package deserialization error for {key:?}, got {err}")
                    })?,
                );
            }
        }

        Ok(packages)
    }

    fn install(
        packages: &BTreeMap<String, Self::Options>,
        _: bool,
        _: &Self::Config,
    ) -> Result<()> {
        for (package, options) in packages {
            let version = options
                .requested_version
                .as_ref()
                .map(|requested_version| format!("@{requested_version}"));
            run_command(
                ["mise"]
                    .into_iter()
                    .chain(if options.active.unwrap_or(true) {
                        ["use"]
                    } else {
                        ["install"]
                    })
                    .chain(["--global", "--yes"])
                    .chain([package.as_str()])
                    .chain(version.iter().map(|s| s.as_str())),
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
        // mise doesn't have a direct cache clean command
        Ok(())
    }

    fn version(_: &Self::Config) -> Result<String> {
        run_command_for_stdout(["mise", "--version"], Perms::Same, false)
    }
}
