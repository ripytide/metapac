use std::collections::BTreeSet;
use std::fs::{self, File, read_to_string};
use std::path::Path;
use std::str::FromStr;

use color_eyre::Result;
use color_eyre::eyre::{Context, ContextCompat, Ok, eyre};
use dialoguer::Confirm;
use strum::IntoEnumIterator;
use toml_edit::{DocumentMut, Entry, Item, Value};

use crate::cli::{BackendsCommand, CleanCacheCommand};
use crate::config::resolve_config_dir;
use crate::prelude::*;

impl MainArguments {
    pub fn run(self) -> Result<()> {
        let hostname = if let Some(x) = self.hostname {
            x
        } else {
            hostname::get()?
                .into_string()
                .or(Err(eyre!("getting hostname")))?
        };

        let config_dir = if let Some(x) = self.config_dir {
            x
        } else {
            resolve_config_dir().wrap_err("determining the default metapac config directory")?
        };

        let group_dir = config_dir.join("groups/");

        let config = Config::load(&config_dir).wrap_err("loading config file")?;

        let enabled_backends = config.enabled_backends(&hostname);

        if enabled_backends.is_empty() {
            log::warn!("no backends found in the enabled_backends config")
        }

        let group_files = config
            .group_files(&group_dir, &hostname)
            .wrap_err("finding group files")?;
        let groups =
            Groups::load(&group_files).wrap_err("loading package options from group files")?;

        let mut required = groups.to_packages()?;

        macro_rules! x {
            ($(($upper_backend:ident, $lower_backend:ident)),*) => {
                $(
                    if !enabled_backends.contains(&AnyBackend::$upper_backend) {
                        if !required.$lower_backend.is_empty() {
                            log::warn!("ignoring {} packages from all group files as the backend was not found in the `enabled_backends` config", AnyBackend::$upper_backend);
                            required.$lower_backend = Default::default();
                        }
                    }
                )*
            }
        }
        apply_backends!(x);

        macro_rules! x {
            ($(($upper_backend:ident, $lower_backend:ident)),*) => {
                $(
                    let are_valid_packages = $upper_backend::are_packages_valid(&required.to_package_ids().$lower_backend, &config.backends.$lower_backend);

                    let invalid_packages = are_valid_packages
                        .iter()
                        .filter_map(|(x, y)| if *y == Some(false) { Some(x) } else { None })
                        .collect::<BTreeSet<_>>();

                    if !invalid_packages.is_empty() {
                        let first_part = format!("the following packages for the {} backend are invalid: {invalid_packages:?}, please fix them, or remove them from your group files", AnyBackend::$upper_backend);
                        let second_part = <$upper_backend as Backend>::invalid_package_help_text();

                        return Err(eyre!("{first_part}\n\n{second_part}"));
                    }
                )*
            }
        }
        apply_backends!(x);

        match self.subcommand {
            MainSubcommand::Add(add) => add.run(&group_dir, &groups),
            MainSubcommand::Remove(remove) => remove.run(&groups),
            MainSubcommand::Install(install) => install.run(&group_dir, &groups, &config.backends),
            MainSubcommand::Uninstall(uninstall) => uninstall.run(&groups, &config.backends),
            MainSubcommand::Update(update) => update.run(&config.backends),
            MainSubcommand::UpdateAll(update_all) => {
                update_all.run(&enabled_backends, &config.backends)
            }
            MainSubcommand::Clean(clean) => {
                clean.run(&required, &enabled_backends, &config.backends)
            }
            MainSubcommand::Sync(sync) => sync.run(&required, &enabled_backends, &config.backends),
            MainSubcommand::Unmanaged(unmanaged) => {
                unmanaged.run(&required, &enabled_backends, &config.backends)
            }
            MainSubcommand::Backends(backends) => backends.run(&config.backends),
            MainSubcommand::CleanCache(clean_cache) => {
                clean_cache.run(&enabled_backends, &config.backends)
            }
        }
    }
}

impl AddCommand {
    fn run(self, group_dir: &Path, groups: &Groups) -> Result<()> {
        let packages = package_vec_to_btreeset(self.packages);

        let packages = packages.iter().filter(|package| {
            let containing_group_files = groups.contains(self.backend, package);

            if !containing_group_files.is_empty() {
                log::warn!("the {package:?} package for the {} backend is already present in the {containing_group_files:?} group files, so this package has been ignored", self.backend);

                false
            } else {
                true
            }
        });

        let group_file = group_dir.join(&self.group).with_extension("toml");

        if !group_file.is_file() {
            File::create_new(&group_file).wrap_err(eyre!(
                "creating an empty group file {}@{group_file:?}",
                &self.group,
            ))?;
        }

        let file_contents = read_to_string(&group_file)
            .wrap_err(eyre!("reading group file {}@{group_file:?}", &self.group))?;

        let mut doc = file_contents
            .parse::<DocumentMut>()
            .wrap_err(eyre!("parsing group file {}@{group_file:?}", &self.group))?;

        let entry = doc.entry(&self.backend.to_string().to_lowercase());
        match entry {
            Entry::Vacant(item) => {
                item.insert(Item::Value(Value::Array(toml_edit::Array::from_iter(
                    packages.clone(),
                ))));
            }
            Entry::Occupied(mut item) => {
                item.get_mut()
                    .as_array_mut()
                    .wrap_err(eyre!(
                        "the {} backend in the {group_file:?} group file has a non-array value",
                        self.backend
                    ))?
                    .extend(packages);
            }
        }

        fs::write(group_file.clone(), doc.to_string())
            .wrap_err(eyre!("writing back modified group file {group_file:?}"))?;

        Ok(())
    }
}

impl RemoveCommand {
    fn run(self, groups: &Groups) -> Result<()> {
        let packages = package_vec_to_btreeset(self.packages);

        for package in packages {
            let containing_group_files = groups.contains(self.backend, &package);
            if !containing_group_files.is_empty() {
                for group_file in containing_group_files {
                    let file_contents = read_to_string(&group_file)
                        .wrap_err(eyre!("reading group file {group_file:?}"))?;

                    let mut doc = file_contents
                        .parse::<DocumentMut>()
                        .wrap_err(eyre!("parsing group file {group_file:?}"))?;

                    let packages = doc
                        .get_mut(&self.backend.to_string().to_lowercase())
                        .unwrap()
                        .as_array_mut()
                        .wrap_err(eyre!(
                            "the {} backend in the {group_file:?} group file has a non-array value",
                            self.backend
                        ))?;

                    packages.retain(|x| match x {
                        Value::String(formatted) => package != formatted.clone().into_value(),
                        Value::InlineTable(inline_table) => {
                            package != inline_table.get("package").unwrap().as_str().unwrap()
                        }
                        _ => unreachable!(),
                    });

                    fs::write(group_file.clone(), doc.to_string())
                        .wrap_err(eyre!("writing back modified group file {group_file:?}"))?;
                }
            } else {
                log::warn!(
                    "the {} package for the {} backend is not in any of the active group files",
                    package,
                    self.backend
                );
            }
        }

        Ok(())
    }
}

impl InstallCommand {
    fn run(
        self,
        group_dir: &Path,
        groups: &Groups,
        backend_configs: &BackendConfigs,
    ) -> Result<()> {
        let packages = package_vec_to_btreeset(self.packages);

        macro_rules! x {
            ($(($upper_backend:ident, $lower_backend:ident)),*) => {
                match self.backend {
                    $(
                        AnyBackend::$upper_backend => {
                            $upper_backend::install(&packages.iter().cloned().map(|x| (x, Default::default())).collect(), self.no_confirm, &backend_configs.$lower_backend)?;
                        },
                    )*
                }
            };
        }
        apply_backends!(x);

        AddCommand {
            backend: self.backend,
            packages: packages.into_iter().collect(),
            group: self.group,
        }
        .run(group_dir, groups)?;

        Ok(())
    }
}

impl UninstallCommand {
    fn run(self, groups: &Groups, backend_configs: &BackendConfigs) -> Result<()> {
        let packages = package_vec_to_btreeset(self.packages);

        macro_rules! x {
            ($(($upper_backend:ident, $lower_backend:ident)),*) => {
                match self.backend {
                    $(
                        AnyBackend::$upper_backend => {
                            $upper_backend::uninstall(&packages, self.no_confirm, &backend_configs.$lower_backend)?;
                        },
                    )*
                }
            };
        }
        apply_backends!(x);

        RemoveCommand {
            backend: self.backend,
            packages: packages.into_iter().collect(),
        }
        .run(groups)?;

        Ok(())
    }
}

impl UpdateCommand {
    fn run(self, backend_configs: &BackendConfigs) -> Result<()> {
        let packages = package_vec_to_btreeset(self.packages);

        self.backend
            .update(&packages, self.no_confirm, backend_configs)
    }
}

impl UpdateAllCommand {
    fn run(
        self,
        enabled_backends: &BTreeSet<AnyBackend>,
        backend_configs: &BackendConfigs,
    ) -> Result<()> {
        let backends = parse_backends(&self.backends, enabled_backends)?;

        for backend in backends.iter() {
            log::info!("updating all packages for {backend} backend");

            backend.update_all(self.no_confirm, backend_configs)?
        }

        Ok(())
    }
}

impl CleanCommand {
    fn run(
        self,
        required: &GroupFilePackages,
        enabled_backends: &BTreeSet<AnyBackend>,
        backend_configs: &BackendConfigs,
    ) -> Result<()> {
        let installed = installed(enabled_backends, backend_configs)?;
        let unmanaged = unmanaged(required, &installed)?;

        if unmanaged.is_empty() {
            log::info!("nothing to clean since there are no unmanaged packages");
            return Ok(());
        }

        print!(
            "{}",
            &unmanaged
                .to_group_file_packages()
                .to_raw()
                .to_string_pretty()?
        );

        if self.no_confirm {
            log::info!("proceeding to uninstall packages without confirmation");
        } else if !Confirm::new()
            .with_prompt("these packages will be uninstalled, do you want to continue?")
            .default(true)
            .show_default(true)
            .interact()
            .wrap_err("getting user confirmation")?
        {
            return Ok(());
        }

        unmanaged.uninstall(self.no_confirm, backend_configs)
    }
}

impl SyncCommand {
    fn run(
        self,
        required: &GroupFilePackages,
        enabled_backends: &BTreeSet<AnyBackend>,
        backend_configs: &BackendConfigs,
    ) -> Result<()> {
        let missing = missing(required, enabled_backends, backend_configs)?;

        if missing.is_empty() {
            log::info!("nothing to install as there are no missing packages");
        }

        if !missing.is_empty() {
            print!("{}", &missing.to_raw().to_string_pretty()?);
        }

        if self.no_confirm {
            log::info!("proceeding to install packages without confirmation");
        } else if !missing.is_empty()
            && !Confirm::new()
                .with_prompt("these packages will be installed, do you want to continue?")
                .default(true)
                .show_default(true)
                .interact()
                .wrap_err("getting user confirmation")?
        {
            return Ok(());
        }

        macro_rules! x {
            ($(($upper_backend:ident, $lower_backend:ident)),*) => {
                $(
                    if enabled_backends.contains(&AnyBackend::$upper_backend) {
                        for package in required.$lower_backend.values() {
                            package.run_before_sync()?;
                        }
                    }
                )*
            };
        }
        apply_backends!(x);

        macro_rules! x {
            ($(($upper_backend:ident, $lower_backend:ident)),*) => {
                $(
                    if enabled_backends.contains(&AnyBackend::$upper_backend) {
                        for package in missing.$lower_backend.values() {
                            package.run_before_install()?;
                        }
                    }
                )*
            };
        }
        apply_backends!(x);

        missing
            .to_packages()
            .install(self.no_confirm, backend_configs)?;

        macro_rules! x {
            ($(($upper_backend:ident, $lower_backend:ident)),*) => {
                $(
                    if enabled_backends.contains(&AnyBackend::$upper_backend) {
                        for package in missing.$lower_backend.values() {
                            package.run_after_install()?;
                        }
                    }
                )*
            };
        }
        apply_backends!(x);

        macro_rules! x {
            ($(($upper_backend:ident, $lower_backend:ident)),*) => {
                $(
                    if enabled_backends.contains(&AnyBackend::$upper_backend) {
                        for package in required.$lower_backend.values() {
                            package.run_after_sync()?;
                        }
                    }
                )*
            };
        }
        apply_backends!(x);

        Ok(())
    }
}

impl UnmanagedCommand {
    fn run(
        self,
        required: &GroupFilePackages,
        enabled_backends: &BTreeSet<AnyBackend>,
        backend_configs: &BackendConfigs,
    ) -> Result<()> {
        let installed = installed(enabled_backends, backend_configs)?;
        let unmanaged = unmanaged(required, &installed)?;

        if unmanaged.is_empty() {
            log::info!("no unmanaged packages");
        } else {
            print!(
                "{}",
                &unmanaged
                    .to_group_file_packages()
                    .to_raw()
                    .to_string_pretty()?
            );
        }

        Ok(())
    }
}

impl BackendsCommand {
    fn run(self, backend_configs: &BackendConfigs) -> Result<()> {
        for backend in AnyBackend::iter() {
            println!(
                "{backend}: {}",
                backend
                    .version(backend_configs)
                    .as_deref()
                    .unwrap_or("Not Found")
                    .trim()
            );
        }

        Ok(())
    }
}

impl CleanCacheCommand {
    fn run(
        &self,
        enabled_backends: &BTreeSet<AnyBackend>,
        backend_configs: &BackendConfigs,
    ) -> Result<()> {
        let backends = parse_backends(&self.backends, enabled_backends)?;

        for backend in backends.iter() {
            log::info!("cleaning cache for {backend} backend");

            backend.clean_cache(backend_configs)?
        }

        Ok(())
    }
}

fn installed(
    enabled_backends: &BTreeSet<AnyBackend>,
    backend_configs: &BackendConfigs,
) -> Result<Packages> {
    macro_rules! x {
        ($(($upper_backend:ident, $lower_backend:ident)),*) => {
            Packages {
                $(
                    $lower_backend:
                        if enabled_backends.contains(&AnyBackend::$upper_backend) {
                            $upper_backend::get_installed(&backend_configs.$lower_backend)?
                        } else {
                            Default::default()
                        },
                )*
            }
        };
    }
    Ok(apply_backends!(x))
}
fn unmanaged(required: &GroupFilePackages, installed: &Packages) -> Result<Packages> {
    let mut output = Packages::default();

    macro_rules! x {
        ($(($upper_backend:ident, $lower_backend:ident)),*) => {
            $(
                for (package_id, package) in installed.$lower_backend.iter() {
                    if (!required.$lower_backend.contains_key(package_id)) {
                        output.$lower_backend.insert(package_id.to_string(), package.clone());
                    }
                }
            )*
        };
    }
    apply_backends!(x);

    Ok(output)
}
fn missing(
    required: &GroupFilePackages,
    enabled_backends: &BTreeSet<AnyBackend>,
    backend_configs: &BackendConfigs,
) -> Result<GroupFilePackages> {
    let installed = installed(enabled_backends, backend_configs)?;

    let mut missing = GroupFilePackages::default();

    macro_rules! x {
        ($(($upper_backend:ident, $lower_backend:ident)),*) => {
            $(
                for (package_id, required_package) in required.$lower_backend.iter() {
                    if !installed.$lower_backend.contains_key(package_id) {
                        missing.$lower_backend.insert(package_id.clone(), required_package.clone());
                    }
                }
            )*
        };
    }
    apply_backends!(x);

    Ok(missing)
}
fn package_vec_to_btreeset(vec: Vec<String>) -> BTreeSet<String> {
    let mut packages = BTreeSet::new();

    for package in vec {
        if !packages.insert(package.clone()) {
            log::warn!("duplicate package {package}, ignoring");
        }
    }

    packages
}
fn parse_backends(
    backends: &Vec<String>,
    enabled_backends: &BTreeSet<AnyBackend>,
) -> Result<BTreeSet<AnyBackend>> {
    if backends.is_empty() {
        Ok(enabled_backends.clone())
    } else if backends == &Vec::from(["all".to_string()]) {
        Ok(AnyBackend::iter().collect())
    } else {
        backends.iter().map(|x|
            AnyBackend::from_str(x)
                .or(Err(eyre!("{x:?} is not a valid backend, run `metapac backends` to see a list of valid backends. Or pass `all` by itself to enable all backends.")))
        ).collect::<Result<BTreeSet<AnyBackend>, _>>()
    }
}
