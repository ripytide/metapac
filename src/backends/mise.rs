use std::collections::{BTreeMap, BTreeSet};

use color_eyre::Result;
use serde::{Deserialize, Serialize};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Mise;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MiseOptions {}

impl Backend for Mise {
    type Options = MiseOptions;

    fn expand_group_packages(
        packages: BTreeMap<String, Package<Self::Options>>,
        _: &Config,
    ) -> Result<BTreeMap<String, Package<Self::Options>>> {
        Ok(packages)
    }

    fn query(config: &Config) -> Result<BTreeMap<String, Self::Options>> {
        if Self::version(config).is_err() {
            return Ok(BTreeMap::new());
        }

        let output = run_command_for_stdout(
            ["mise", "list", "--installed"],
            Perms::Same,
            true,
        )?;

        let mut packages = BTreeMap::new();
        
        for line in output.lines() {
            // Parse mise list output which typically shows: tool@version
            if let Some(tool) = line.split_whitespace().next() {
                // Extract tool name from tool@version format
                let tool_name = if let Some(at_pos) = tool.find('@') {
                    &tool[..at_pos]
                } else {
                    tool
                };
                
                if !tool_name.is_empty() {
                    packages.insert(tool_name.to_string(), Self::Options {});
                }
            }
        }

        Ok(packages)
    }

    fn install(packages: &BTreeMap<String, Self::Options>, _: bool, _: &Config) -> Result<()> {
        for package in packages.keys() {
            run_command(["mise", "install", package], Perms::Same)?;
        }

        Ok(())
    }

    fn uninstall(packages: &BTreeSet<String>, _: bool, _: &Config) -> Result<()> {
        for package in packages {
            run_command(["mise", "uninstall", package], Perms::Same)?;
        }

        Ok(())
    }

    fn update(packages: &BTreeSet<String>, _: bool, _: &Config) -> Result<()> {
        for package in packages {
            run_command(["mise", "upgrade", package], Perms::Same)?;
        }

        Ok(())
    }

    fn update_all(_: bool, _: &Config) -> Result<()> {
        run_command(["mise", "upgrade"], Perms::Same)
    }

    fn clean_cache(_: &Config) -> Result<()> {
        // mise doesn't have a direct cache clean command, so we'll just return Ok
        Ok(())
    }

    fn version(_: &Config) -> Result<String> {
        run_command_for_stdout(["mise", "--version"], Perms::Same, false)
    }
}