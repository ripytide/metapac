use std::collections::BTreeMap;
use std::collections::BTreeSet;

use color_eyre::Result;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;

use crate::cmd::run_command;
use crate::cmd::run_command_for_stdout;
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct VsCode;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VsCodeOptions {}

impl Backend for VsCode {
    type Options = VsCodeOptions;

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

        let names = run_command_for_stdout(
            [config.vscode.variant.as_command(), "--list-extensions"],
            Perms::Same,
            true,
        )?
        .lines()
        .map(|x| (x.to_string(), Self::Options {}))
        .collect();

        Ok(names)
    }

    fn install(packages: &BTreeMap<String, Self::Options>, _: bool, config: &Config) -> Result<()> {
        for package in packages.keys() {
            run_command(
                [
                    config.vscode.variant.as_command(),
                    "--install-extension",
                    package,
                ],
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn uninstall(packages: &BTreeSet<String>, _: bool, config: &Config) -> Result<()> {
        for package in packages {
            run_command(
                [
                    config.vscode.variant.as_command(),
                    "--uninstall-extension",
                    package,
                ],
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn update(packages: &BTreeSet<String>, _: bool, config: &Config) -> Result<()> {
        for package in packages {
            run_command(
                [
                    config.vscode_variant.as_command(),
                    "--install-extension",
                    package,
                ],
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn update_all(no_confirm: bool, config: &Config) -> Result<()> {
        let packages = Self::query(config)?;
        Self::update(
            &packages.keys().map(String::from).collect(),
            no_confirm,
            config,
        )
    }

    fn clean_cache(_: &Config) -> Result<()> {
        Ok(())
    }

    fn version(config: &Config) -> Result<String> {
        run_command_for_stdout(
            [config.vscode.variant.as_command(), "--version"],
            Perms::Same,
            false,
        )
        .map(|x| x.lines().join(" "))
    }
}
