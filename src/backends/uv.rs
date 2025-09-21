use std::collections::BTreeMap;
use std::collections::BTreeSet;

use color_eyre::Result;
use serde::Deserialize;
use serde::Serialize;

use crate::cmd::run_command;
use crate::cmd::run_command_for_stdout;
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Uv;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UvOptions {
    #[serde(default)]
    python: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct UvConfig {}

impl Backend for Uv {
    type Options = UvOptions;
    type Config = UvConfig;

    fn invalid_package_help_text() -> String {
        String::new()
    }

    fn are_valid_packages(
        packages: &BTreeSet<String>,
        _: &Config,
    ) -> BTreeMap<String, Option<bool>> {
        packages.iter().map(|x| (x.to_string(), None)).collect()
    }

    fn get_installed(config: &Self::Config) -> Result<BTreeMap<String, Self::Options>> {
        if Self::version(config).is_err() {
            return Ok(BTreeMap::new());
        }

        let names = run_command_for_stdout(
            ["uv", "tool", "list", "--color", "never"],
            Perms::Same,
            true,
        )?
        .lines()
        .filter(|x| !x.starts_with("-"))
        .map(|x| x.split(" ").next().unwrap().to_string())
        .map(|x| (x, Self::Options { python: None }))
        .collect();

        Ok(names)
    }

    fn install(
        packages: &BTreeMap<String, Self::Options>,
        _: bool,
        _: &Self::Config,
    ) -> Result<()> {
        for (package, options) in packages {
            run_command(
                ["uv", "tool", "install"]
                    .into_iter()
                    .chain(
                        Some("--python")
                            .into_iter()
                            .filter(|_| options.python.is_some()),
                    )
                    .chain(options.python.as_deref())
                    .chain([package.as_str()]),
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn uninstall(packages: &BTreeSet<String>, _: bool, _: &Self::Config) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["uv", "tool", "uninstall"]
                    .into_iter()
                    .chain(packages.iter().map(String::as_str)),
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn update(packages: &BTreeSet<String>, _: bool, _: &Self::Config) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["uv", "tool", "upgrade"]
                    .into_iter()
                    .chain(packages.iter().map(String::as_str)),
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn update_all(_: bool, _: &Self::Config) -> Result<()> {
        run_command(["uv", "tool", "upgrade", "--all"], Perms::Same)
    }

    fn clean_cache(_: &Self::Config) -> Result<()> {
        Ok(())
    }

    fn version(_: &Self::Config) -> Result<String> {
        run_command_for_stdout(["uv", "--version"], Perms::Same, false)
    }
}
