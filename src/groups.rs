use crate::prelude::*;
use color_eyre::{
    eyre::{eyre, Context, ContextCompat},
    Result,
};
use toml::{Table, Value};

use std::{
    collections::BTreeMap,
    fs::read_to_string,
    ops::AddAssign,
    path::{Path, PathBuf},
};

#[derive(Debug, Default, derive_more::Deref, derive_more::DerefMut)]
pub struct Groups(BTreeMap<PathBuf, RawInstallOptions>);

impl Groups {
    pub fn contains(&self, backend: AnyBackend, package: &str) -> Vec<PathBuf> {
        let mut result = Vec::new();
        for (group_file, raw_install_options) in self.0.iter() {
            if raw_install_options
                .to_raw_package_ids()
                .contains(backend, package)
            {
                result.push(group_file.clone());
            }
        }
        result
    }

    pub fn to_install_options(&self) -> InstallOptions {
        let mut reoriented: BTreeMap<(AnyBackend, String), BTreeMap<PathBuf, u32>> =
            BTreeMap::new();

        for (group_file, raw_install_options) in self.iter() {
            let raw_package_ids = raw_install_options.to_raw_package_ids();

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
            apply_public_backends!(x);
        }

        //warn the user about duplicated packages and output a deduplicated InstallOptions
        for ((backend, package), group_files) in reoriented.iter() {
            if group_files.len() > 1 {
                log::warn!("duplicate {package:?} package in group files: {group_files:?} for the {backend} backend");
                log::warn!("only one of the duplicated will be used which could may cause unintended behaviour if the duplicates have different install options");
            }
        }

        let mut merged_raw_install_options = RawInstallOptions::default();
        for mut raw_install_options in self.values().cloned() {
            merged_raw_install_options.append(&mut raw_install_options);
        }

        let mut install_options = InstallOptions::default();
        macro_rules! x {
            ($(($upper_backend:ident, $lower_backend:ident)),*) => {
                $(
                    install_options.$lower_backend = merged_raw_install_options.$lower_backend.into_iter().collect();
                )*
            };
        }
        apply_public_backends!(x);

        install_options
    }

    pub fn load(group_dir: &Path, hostname: &str, config: &Config) -> Result<Groups> {
        if !group_dir.is_dir() {
            log::warn!("the groups directory: {group_dir:?}, was not found, assuming there are no group files. If this was intentional please create an empty groups folder.");

            return Ok(Groups::default());
        }

        let group_files = if config.hostname_groups_enabled {
            let group_names = config.hostname_groups.get(hostname).wrap_err(eyre!(
                "no hostname entry in the hostname_groups config for the hostname: {hostname}"
            ))?;

            group_names
                .iter()
                .map(|group_name| group_dir.join(group_name).with_extension("toml"))
                .collect::<Vec<_>>()
        } else {
            walkdir::WalkDir::new(group_dir)
                .follow_links(true)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|x| !x.file_type().is_dir())
                .map(|x| x.path().to_path_buf())
                .collect::<Vec<_>>()
        };

        let mut groups = Self::default();

        for group_file in group_files {
            let file_contents =
                read_to_string(&group_file).wrap_err(eyre!("reading group file {group_file:?}"))?;

            let raw_install_options = parse_group_file(&group_file, &file_contents)
                .wrap_err(eyre!("parsing group file {group_file:?}"))?;

            groups.insert(group_file, raw_install_options);
        }

        Ok(groups)
    }
}

fn parse_group_file(group_file: &Path, contents: &str) -> Result<RawInstallOptions> {
    let mut raw_install_options = RawInstallOptions::default();

    let toml = toml::from_str::<Table>(contents)?;

    for (key, value) in toml.iter() {
        raw_install_options.append(&mut parse_toml_key_value(group_file, key, value)?);
    }

    Ok(raw_install_options)
}

fn parse_toml_key_value(group_file: &Path, key: &str, value: &Value) -> Result<RawInstallOptions> {
    macro_rules! x {
        ($(($upper_backend:ident, $lower_backend:ident)),*) => {
            $(
                if key.to_lowercase() == $upper_backend.to_string().to_lowercase() {
                    let mut raw_install_options = RawInstallOptions::default();

                    let packages = value.as_array().ok_or(
                        eyre!("the {} backend in the {group_file:?} group file has a non-array value", $upper_backend)
                    )?;

                    for package in packages {
                        let (package, package_install_options) =
                            match package {
                                toml::Value::String(x) => (x.to_string(), Default::default()),
                                toml::Value::Table(x) => (
                                    x.clone().try_into::<StringPackageStruct>()?.package,
                                    x.clone().try_into()?,
                                ),
                                _ => return Err(eyre!("the {} backend in the {group_file:?} group file has a package which is neither a string or a table", $upper_backend)),
                            };

                        raw_install_options.$lower_backend.push((package, package_install_options));
                    }

                    return Ok(raw_install_options);
                }
            )*
        };
    }
    apply_public_backends!(x);

    log::warn!("unrecognised backend: {key:?} in group file: {group_file:?}");

    Ok(RawInstallOptions::default())
}
