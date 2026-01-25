use std::collections::{BTreeMap, BTreeSet};
use std::io::ErrorKind::NotFound;

use color_eyre::Result;
use color_eyre::eyre::{Context, eyre};
use serde::{Deserialize, Serialize};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Cargo;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct CargoConfig {
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub binstall: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CargoPackageOptions {
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    git: Option<String>,
    #[serde(default)]
    all_features: Option<bool>,
    #[serde(default)]
    no_default_features: Option<bool>,
    #[serde(default)]
    features: Vec<String>,
    #[serde(default)]
    locked: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CargoRepoOptions {}

impl Backend for Cargo {
    type Config = CargoConfig;
    type PackageOptions = CargoPackageOptions;
    type RepoOptions = CargoRepoOptions;

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

        let toml_file = home::cargo_home()
            .wrap_err("getting the cargo home directory")?
            .join(".crates.toml");

        match std::fs::read_to_string(&toml_file) {
            Ok(contents) => extract_packages(&contents),
            Err(err) if err.kind() == NotFound => {
                log::warn!(
                    "no .crates.toml file found for cargo, assuming no crates installed yet"
                );
                Ok(BTreeMap::new())
            }
            Err(err) => Err(err.into()),
        }
    }

    fn install_packages(
        packages: &BTreeMap<String, Self::PackageOptions>,
        _: bool,
        config: &Self::Config,
    ) -> Result<()> {
        for (package, options) in packages {
            run_command(
                ["cargo"]
                    .into_iter()
                    .chain(if !config.binstall {
                        vec!["install"]
                    } else {
                        vec!["binstall", "--no-confirm"]
                    })
                    .chain(
                        Some("--locked")
                            .into_iter()
                            .filter(|_| options.locked.unwrap_or(config.locked)),
                    )
                    .chain(Some("--git").into_iter().filter(|_| options.git.is_some()))
                    .chain(options.git.as_deref())
                    .chain(
                        Some("--all-features")
                            .into_iter()
                            .filter(|_| options.all_features.is_some_and(|x| x)),
                    )
                    .chain(
                        Some("--no-default-features")
                            .into_iter()
                            .filter(|_| options.no_default_features.is_some_and(|x| x)),
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

    fn uninstall_packages(packages: &BTreeSet<String>, _: bool, _: &Self::Config) -> Result<()> {
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

    fn update_packages(
        packages: &BTreeSet<String>,
        no_confirm: bool,
        config: &Self::Config,
    ) -> Result<()> {
        // upstream issue in case cargo ever implements a simpler way to do this
        // https://github.com/rust-lang/cargo/issues/9527

        let installed = Self::get_installed_packages(config)?;
        let installed_names = installed.keys().map(String::from).collect();

        let difference = packages
            .difference(&installed_names)
            .collect::<BTreeSet<_>>();

        if !difference.is_empty() {
            return Err(eyre!("{difference:?} packages are not installed"));
        }

        let mut install_options = installed
            .clone()
            .into_iter()
            .filter(|(x, _)| packages.contains(x))
            .collect::<BTreeMap<String, CargoPackageOptions>>();

        for options in install_options.values_mut() {
            options.locked = Some(config.locked);
        }

        Self::install_packages(&install_options, no_confirm, config)
    }

    fn update_all_packages(no_confirm: bool, config: &Self::Config) -> Result<()> {
        // upstream issue in case cargo ever implements a simpler way to do this
        // https://github.com/rust-lang/cargo/issues/9527

        let install_options = Self::get_installed_packages(config)?;

        Self::install_packages(&install_options, no_confirm, config)
    }

    fn clean_cache(_: &Self::Config) -> Result<()> {
        run_command_for_stdout(["cargo-cache", "-V"], Perms::Same, StdErr::Show).map_or(Ok(()), |_| {
            run_command(["cargo", "cache", "-a"], Perms::Same)
        })
    }

    fn get_installed_repos(_: &Self::Config) -> Result<BTreeMap<String, Self::RepoOptions>> {
        Ok(BTreeMap::new())
    }

    fn add_repos(_: &BTreeMap<String, Self::RepoOptions>, _: bool, _: &Self::Config) -> Result<()> {
        Err(eyre!("unimplemented"))
    }

    fn remove_repos(_: &BTreeSet<String>, _: bool, _: &Self::Config) -> Result<()> {
        Err(eyre!("unimplemented"))
    }

    fn version(_: &Self::Config) -> Result<String> {
        run_command_for_stdout(["cargo", "--version"], Perms::Same, StdErr::Show)
    }
}

fn extract_packages(contents: &str) -> Result<BTreeMap<String, CargoPackageOptions>> {
    let toml: toml::Table =
        toml::from_str(contents).wrap_err("parsing TOML from .crates.toml file")?;

    let v1_section = toml
        .get("v1")
        .ok_or(eyre!("missing 'v1' section in .crates.toml"))?
        .as_table()
        .ok_or(eyre!("'v1' section should be a table"))?;

    let mut packages = BTreeMap::new();

    for (key, _value) in v1_section {
        // Key format: "package_name version (source)"
        // Extract package name (everything before the first space)
        if let Some((package_name, version_source)) = key.split_once(' ') {
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

            packages.insert(
                package_name.to_string(),
                CargoPackageOptions {
                    version: Some(version.to_string()),
                    git,
                    // All of these are not specified
                    all_features: None,
                    no_default_features: None,
                    features: Vec::new(),
                    locked: None,
                },
            );
        }
    }

    Ok(packages)
}
