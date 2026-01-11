use crate::prelude::*;
use color_eyre::{
    Result,
    eyre::{Context, eyre},
};
use toml::{Table, Value};

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::read_to_string,
    ops::AddAssign,
    path::{Path, PathBuf},
};

#[derive(Debug, Default, derive_more::Deref, derive_more::DerefMut)]
pub struct Groups(BTreeMap<PathBuf, AllRawComplexBackendItems>);

impl Groups {
    pub fn contains(&self, backend: AnyBackend, package: &str) -> BTreeSet<PathBuf> {
        let mut results = BTreeSet::new();
        for (group_file, all_items) in self.0.iter() {
            macro_rules! x {
                ($(($upper_backend:ident, $lower_backend:ident)),*) => {
                    match backend {
                        $(
                            AnyBackend::$upper_backend => {
                                if all_items.$lower_backend.packages.iter().any(|x| x.name == package) {
                                    results.insert(group_file.clone());
                                }
                            },
                        )*
                    }
                };
            }
            apply_backends!(x);
        }
        results
    }

    pub fn to_combined(&self) -> AllComplexBackendItems {
        let mut reoriented: BTreeMap<(AnyBackend, String), BTreeMap<PathBuf, u32>> =
            BTreeMap::new();

        for (group_file, all_raw_complex_backend_items) in self.iter() {
            macro_rules! x {
                ($(($upper_backend:ident, $lower_backend:ident)),*) => {
                    $(
                        for package in all_raw_complex_backend_items.$lower_backend.packages.iter() {
                            reoriented
                                .entry((AnyBackend::$upper_backend, package.name.clone()))
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

        let mut merged_raw_packages = AllRawComplexBackendItems::default();
        for mut raw_packages in self.values().cloned() {
            merged_raw_packages.append(&mut raw_packages);
        }

        macro_rules! x {
            ($(($upper_backend:ident, $lower_backend:ident)),*) => {
                AllComplexBackendItems {
                    $(
                        $lower_backend: merged_raw_packages.$lower_backend.to_non_raw(),
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

fn parse_group_file(group_file: &Path, contents: &str) -> Result<AllRawComplexBackendItems> {
    let mut raw_packages = AllRawComplexBackendItems::default();

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
) -> Result<AllRawComplexBackendItems> {
    macro_rules! x {
        ($(($upper_backend:ident, $lower_backend:ident)),*) => {
            $(
                let backend_property = $upper_backend.to_string().to_lowercase();
                if key.to_lowercase() == backend_property {
                    let mut items = AllRawComplexBackendItems::default();

                    let backend = value.as_table().ok_or(
                        eyre!("the {backend_property:?} property in the {group_file:?} group file has a non-table value")
                    )?;

                    for (key, _) in backend.iter() {
                        if key != "packages" && key != "repos" {
                            return Err(eyre!("unrecognised property: \"{backend_property}.{key}\" in group file: {group_file:?}"))
                        }
                    }

                    if let Some(packages) = backend.get("packages") {
                        let packages = packages.as_array().ok_or(
                            eyre!("the \"{backend_property}.packages\" property in the {group_file:?} group file has a non-array value")
                        )?;

                        for package in packages {
                            let package =
                                match package {
                                    toml::Value::String(x) => ComplexItem { name: x.to_string(), options: Default::default(), hooks: Hooks::default() },
                                    toml::Value::Table(x) => x.clone().try_into::<ComplexItem<<$upper_backend as Backend>::PackageOptions>>()?,
                                    _ => return Err(eyre!("the \"{backend_property}.packages\" array in the {group_file:?} group file has a package which is neither a string or a table")),
                                };

                            items.$lower_backend.packages.push(package);
                        }
                    }

                    if let Some(repos) = backend.get("repos") {
                        let repos = repos.as_array().ok_or(
                            eyre!("the \"{backend_property}.repos\" property in the {group_file:?} group file has a non-array value")
                        )?;

                        for repo in repos {
                            let repo =
                                match repo {
                                    toml::Value::String(x) => ComplexItem { name: x.to_string(), options: Default::default(), hooks: Hooks::default() },
                                    toml::Value::Table(x) => x.clone().try_into::<ComplexItem<<$upper_backend as Backend>::RepoOptions>>()?,
                                    _ => return Err(eyre!("the \"{backend_property}.repos\" array in the {group_file:?} group file has a repo which is neither a string or a table")),
                                };

                            items.$lower_backend.repos.push(repo);
                        }
                    }

                    return Ok(items);
                }
            )*
        };
    }
    apply_backends!(x);

    Err(eyre!(
        "unrecognised property: {key:?} in group file: {group_file:?}"
    ))
}
