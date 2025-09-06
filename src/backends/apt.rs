use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Apt;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AptOptions {}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct AptConfig {}

impl Backend for Apt {
    type Options = AptOptions;
    type Config = AptConfig;

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

        // See https://askubuntu.com/questions/2389/how-to-list-manually-installed-packages
        // for a run-down of methods for finding lists of
        // explicit/dependency packages. It doesn't seem as if apt was
        // designed with this use-case in mind so there are lots and
        // lots of different methods all of which seem to have
        // caveats.
        let explicit = run_command_for_stdout(["apt-mark", "showmanual"], Perms::Same, false)?;
        Ok(explicit
            .lines()
            .map(|x| (x.to_string(), Self::Options {}))
            .collect())
    }

    fn install(
        packages: &BTreeMap<String, Self::Options>,
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

    fn uninstall(packages: &BTreeSet<String>, no_confirm: bool, _: &Self::Config) -> Result<()> {
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

    fn update(packages: &BTreeSet<String>, no_confirm: bool, _: &Self::Config) -> Result<()> {
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

    fn update_all(no_confirm: bool, _: &Self::Config) -> Result<()> {
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

    fn version(_: &Self::Config) -> Result<String> {
        run_command_for_stdout(["apt", "--version"], Perms::Same, false)
    }
}
