use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::cmd::run_command;
use crate::cmd::run_command_for_stdout;
use crate::prelude::*;
use color_eyre::Result;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct VsCode;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VsCodeOptions {}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct VsCodeConfig {
    #[serde(default)]
    pub variant: VsCodeVariant,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VsCodeVariant {
    #[default]
    Code,
    Codium,
}
impl VsCodeVariant {
    pub fn as_command(&self) -> &'static str {
        match self {
            VsCodeVariant::Code => "code",
            VsCodeVariant::Codium => "codium",
        }
    }
}

impl Backend for VsCode {
    type Options = VsCodeOptions;
    type Config = VsCodeConfig;

    fn invalid_package_help_text() -> String {
        String::new()
    }

    fn are_valid_packages(
        packages: &BTreeSet<String>,
        _: &Config,
    ) -> BTreeMap<String, Option<bool>> {
        packages.iter().map(|x| (x.to_string(), None)).collect()
    }

    fn query(config: &Self::Config) -> Result<BTreeMap<String, Self::Options>> {
        if Self::version(config).is_err() {
            return Ok(BTreeMap::new());
        }

        let names = run_command_for_stdout(
            [config.variant.as_command(), "--list-extensions"],
            Perms::Same,
            true,
        )?
        .lines()
        .map(|x| (x.to_string(), Self::Options {}))
        .collect();

        Ok(names)
    }

    fn install(
        packages: &BTreeMap<String, Self::Options>,
        _: bool,
        config: &Self::Config,
    ) -> Result<()> {
        for package in packages.keys() {
            run_command(
                [config.variant.as_command(), "--install-extension", package],
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn uninstall(packages: &BTreeSet<String>, _: bool, config: &Self::Config) -> Result<()> {
        for package in packages {
            run_command(
                [
                    config.variant.as_command(),
                    "--uninstall-extension",
                    package,
                ],
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn update(packages: &BTreeSet<String>, _: bool, config: &Self::Config) -> Result<()> {
        for package in packages {
            run_command(
                [config.variant.as_command(), "--install-extension", package],
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn update_all(no_confirm: bool, config: &Self::Config) -> Result<()> {
        let packages = Self::query(config)?;
        Self::update(
            &packages.keys().map(String::from).collect(),
            no_confirm,
            config,
        )
    }

    fn clean_cache(_: &Self::Config) -> Result<()> {
        Ok(())
    }

    fn version(config: &Self::Config) -> Result<String> {
        run_command_for_stdout(
            [config.variant.as_command(), "--version"],
            Perms::Same,
            false,
        )
        .map(|x| x.lines().join(" "))
    }
}
