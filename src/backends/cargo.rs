use std::collections::{BTreeMap, BTreeSet};
use std::io::ErrorKind::NotFound;

use color_eyre::Result;
use color_eyre::eyre::{Context, eyre};
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use serde_json::Value;

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Cargo;

#[serde_inline_default]
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CargoOptions {
    #[serde_inline_default(CargoOptions::default().version)]
    version: Option<String>,
    #[serde_inline_default(CargoOptions::default().git)]
    git: Option<String>,
    #[serde_inline_default(CargoOptions::default().all_features)]
    all_features: bool,
    #[serde_inline_default(CargoOptions::default().no_default_features)]
    no_default_features: bool,
    #[serde_inline_default(CargoOptions::default().features)]
    features: Vec<String>,
    #[serde_inline_default(CargoOptions::default().locked)]
    locked: Option<bool>,
}

impl Backend for Cargo {
    type Options = CargoOptions;

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

        let file = home::cargo_home()
            .wrap_err("getting the cargo home directory")?
            .join(".crates2.json");

        let contents = match std::fs::read_to_string(file) {
            Ok(string) => string,
            Err(err) if err.kind() == NotFound => {
                log::warn!("no crates file found for cargo, assuming no crates installed yet");
                return Ok(BTreeMap::new());
            }
            Err(err) => return Err(err.into()),
        };

        extract_packages(&contents).wrap_err("extracting packages from crates file")
    }

    fn install(packages: &BTreeMap<String, Self::Options>, _: bool, config: &Config) -> Result<()> {
        for (package, options) in packages {
            run_command(
                ["cargo", "install"]
                    .into_iter()
                    .chain(
                        Some("--locked")
                            .into_iter()
                            .filter(|_| options.locked.unwrap_or(config.cargo.locked)),
                    )
                    .chain(Some("--git").into_iter().filter(|_| options.git.is_some()))
                    .chain(options.git.as_deref())
                    .chain(
                        Some("--all-features")
                            .into_iter()
                            .filter(|_| options.all_features),
                    )
                    .chain(
                        Some("--no-default-features")
                            .into_iter()
                            .filter(|_| options.no_default_features),
                    )
                    .chain(
                        Some("--features")
                            .into_iter()
                            .filter(|_| !options.features.is_empty()),
                    )
                    .chain(options.features.iter().map(String::as_str))
                    .chain([package.as_str()]),
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn uninstall(packages: &BTreeSet<String>, _: bool, _: &Config) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["cargo", "uninstall"]
                    .into_iter()
                    .chain(packages.iter().map(String::as_str)),
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn clean_cache(_: &Config) -> Result<()> {
        run_command_for_stdout(["cargo-cache", "-V"], Perms::Same, false).map_or(Ok(()), |_| {
            run_command(["cargo", "cache", "-a"], Perms::Same)
        })
    }

    fn version(_: &Config) -> Result<String> {
        run_command_for_stdout(["cargo", "--version"], Perms::Same, false)
    }
}

fn extract_packages(contents: &str) -> Result<BTreeMap<String, CargoOptions>> {
    let json: Value = serde_json::from_str(contents).wrap_err("parsing JSON from crates file")?;

    let result: BTreeMap<String, CargoOptions> = json
        .get("installs")
        .ok_or(eyre!("get 'installs' field from json"))?
        .as_object()
        .ok_or(eyre!("getting object"))?
        .into_iter()
        .map(|(name, value)| {
            let value = value.as_object().unwrap();

            let (name, version_source) = name.split_once(' ').unwrap();
            let (version, source) = version_source.split_once(' ').unwrap();

            let git = if source.starts_with("(git+") {
                Some(
                    source.split("+").collect::<Vec<_>>()[1]
                        .split("#")
                        .next()
                        .unwrap()
                        .to_string(),
                )
            } else {
                None
            };

            let all_features = value.get("all_features").unwrap().as_bool().unwrap();
            let no_default_features = value.get("no_default_features").unwrap().as_bool().unwrap();
            let features = value
                .get("features")
                .unwrap()
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.as_str().unwrap().to_string())
                .collect();

            (
                name.to_string(),
                CargoOptions {
                    version: Some(version.to_string()),
                    git,
                    all_features,
                    no_default_features,
                    features,
                    locked: None, //cargo does not track if the install happened with --locked
                },
            )
        })
        .collect();

    Ok(result)
}
