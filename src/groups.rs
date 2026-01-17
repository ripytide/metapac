use crate::prelude::*;
use color_eyre::{
    Result,
    eyre::{Context, eyre},
};
use serde::{Deserialize, Serialize};
use toml::{Table, Value};

use crate::cmd::run_command;

use std::{
    collections::BTreeMap,
    fs::read_to_string,
    ops::AddAssign,
    path::{Path, PathBuf},
};

#[derive(Debug, Default, derive_more::Deref, derive_more::DerefMut)]
pub struct Groups(BTreeMap<PathBuf, RawGroupFile>);

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackendItems<P, R> {
    #[serde(default)]
    packages: Vec<GroupFileItem<P>>,
    #[serde(default)]
    repos: Vec<GroupFileItem<R>>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupFileItem<T> {
    pub name: String,
    #[serde(default)]
    pub options: T,
    #[serde(default)]
    pub hooks: Hooks,
}
impl<T> GroupFileItem<T> {
    pub fn into_options(self) -> T {
        self.options
    }

    pub fn run_before_install(&self) -> Result<()> {
        if let Some(args) = &self.hooks.before_install {
            log::info!("running before_install hook for item: {:?}", self.name);
            run_command(args, Perms::Same)
        } else {
            Ok(())
        }
    }
    pub fn run_after_install(&self) -> Result<()> {
        if let Some(args) = &self.hooks.after_install {
            log::info!("running after_install hook for item: {:?}", self.name);
            run_command(args, Perms::Same)
        } else {
            Ok(())
        }
    }
    pub fn run_after_sync(&self) -> Result<()> {
        if let Some(args) = &self.hooks.after_sync {
            log::info!("running after_sync hook for item: {:?}", self.name);
            run_command(args, Perms::Same)
        } else {
            Ok(())
        }
    }
    pub fn run_before_sync(&self) -> Result<()> {
        if let Some(args) = &self.hooks.before_sync {
            log::info!("running before_sync hook for item: {:?}", self.name);
            run_command(args, Perms::Same)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hooks {
    pub before_install: Option<Vec<String>>,
    pub after_install: Option<Vec<String>>,
    pub after_sync: Option<Vec<String>>,
    pub before_sync: Option<Vec<String>>,
}

impl Groups {
    pub fn contains(&self, backend: AnyBackend, package: &str) -> Vec<PathBuf> {
        let mut result = Vec::new();
        for (group_file, raw_packages) in self.0.iter() {
            if raw_packages.to_raw_package_ids().contains(backend, package) {
                result.push(group_file.clone());
            }
        }
        result
    }

    pub fn to_group_file_packages(&self) -> GroupFilePackages {
        let mut reoriented: BTreeMap<(AnyBackend, String), BTreeMap<PathBuf, u32>> =
            BTreeMap::new();

        for (group_file, raw_packages) in self.iter() {
            let raw_package_ids = raw_packages.to_raw_package_ids();

            macro_rules! x {
                ($(($upper_backend:ident, $lower_backend:ident)),*) => {
                    $(
                        for package in raw_package_ids.$lower_backend {
                            reoriented
                                .entry((AnyBackend::$upper_backend, package.clone()))
                                .or_default()
                                .entry(group_file.clone())
                                .or_default()
                                .add_assign(1);
                        }
                    )*
                };
            }
            apply_backends!(x);
        }

        for ((backend, package), group_files_counts) in reoriented.iter() {
            if group_files_counts.len() > 1 || group_files_counts.values().any(|y| *y > 1) {
                let group_files = group_files_counts.keys().cloned().collect::<Vec<_>>();

                // this is only a warning and not a hard error since there is a valid use-case for
                // repeating shared optional dependencies for better atomnicity when adding and
                // removing packages, see <https://github.com/ripytide/metapac/discussions/149>
                log::warn!(
                    "duplicate package: {package:?} found in group files: {group_files:?} for the {backend} backend"
                );
            }
        }

        let mut merged_raw_packages = RawGroupFilePackages::default();
        for mut raw_packages in self.values().cloned() {
            merged_raw_packages.append(&mut raw_packages);
        }

        macro_rules! x {
            ($(($upper_backend:ident, $lower_backend:ident)),*) => {
                GroupFilePackages {
                    $(
                        $lower_backend: merged_raw_packages.$lower_backend.into_iter().map(|x| (x.package.clone(), x)).collect(),
                    )*
                }
            };
        }
        apply_backends!(x)
    }

    pub fn load(hostname: &str, group_dir: &Path, config: &Config) -> Result<Groups> {
        let group_files = config
            .group_files(group_dir, hostname)
            .wrap_err("finding group files")?;

        let mut groups = Self::default();

        for group_file in group_files.iter() {
            let file_contents =
                read_to_string(group_file).wrap_err(eyre!("reading group file {group_file:?}"))?;

            let raw_packages = parse_group_file(group_file, &file_contents)
                .wrap_err(eyre!("parsing group file {group_file:?}"))?;

            groups.insert(group_file.clone(), raw_packages);
        }

        Ok(groups)
    }
}

fn parse_group_file(group_file: &Path, contents: &str) -> Result<RawGroupFile> {
    let mut raw_packages = RawGroupFile::default();

    let toml = toml::from_str::<Table>(contents)?;

    for (key, value) in toml.iter() {
        raw_packages.append(&mut parse_toml_key_value(group_file, key, value)?);
    }

    Ok(raw_packages)
}

fn parse_toml_key_value(
    group_file: &Path,
    key: &str,
    value: &Value,
) -> Result<RawGroupFile> {
    macro_rules! x {
        ($(($upper_backend:ident, $lower_backend:ident)),*) => {
            $(
                if key.to_lowercase() == $upper_backend.to_string().to_lowercase() {
                    let mut raw_packages = RawGroupFile::default();

                    let packages = value.as_array().ok_or(
                        eyre!("the {} backend in the {group_file:?} group file has a non-array value", $upper_backend)
                    )?;

                    for package in packages {
                        let package =
                            match package {
                                toml::Value::String(x) => GroupFileItem { name: x.to_string(), options: Default::default(), hooks: Hooks::default() },
                                toml::Value::Table(x) => x.clone().try_into::<GroupFileItem<<$upper_backend as Backend>::Options>>()?,
                                _ => return Err(eyre!("the {} backend in the {group_file:?} group file has a package which is neither a string or a table", $upper_backend)),
                            };

                        raw_packages.$lower_backend.push(package);
                    }

                    return Ok(raw_packages);
                }
            )*
        };
    }
    apply_backends!(x);

    log::warn!("unrecognised backend: {key:?} in group file: {group_file:?}");

    Ok(RawGroupFile::default())
}
