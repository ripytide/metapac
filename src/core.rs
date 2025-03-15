use std::collections::BTreeSet;
use std::fs::{self, read_to_string, File};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use color_eyre::eyre::{eyre, Context, ContextCompat, Ok};
use color_eyre::Result;
use dialoguer::Confirm;
use strum::IntoEnumIterator;
use toml_edit::{Array, DocumentMut, Item, Value};

use crate::cli::{BackendsCommand, CleanCacheCommand};
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
            dirs::config_dir()
                .map(|path| path.join("metapac/"))
                .ok_or(eyre!("getting the default metapac config directory"))?
        };

        let group_dir = config_dir.join("groups/");

        let config = Config::load(&config_dir).wrap_err("loading config file")?;
        let group_files = Groups::group_files(&group_dir, &hostname, &config)
            .wrap_err("failed to find group files")?;
        let groups = Groups::load(&group_files)
            .wrap_err("failed to load package install options from groups")?;

        let required = groups.to_options().map_required(&config)?;

        match self.subcommand {
            MainSubcommand::Add(add) => add.run(&group_dir, &group_files, &groups, &config),
            MainSubcommand::Remove(remove) => remove.run(&groups),
            MainSubcommand::Install(install) => {
                install.run(&group_dir, &group_files, &groups, &config)
            }
            MainSubcommand::Uninstall(uninstall) => uninstall.run(&groups, &config),
            MainSubcommand::Clean(clean) => clean.run(&required, &config),
            MainSubcommand::Sync(sync) => sync.run(&required, &config),
            MainSubcommand::Unmanaged(unmanaged) => unmanaged.run(&required, &config),
            MainSubcommand::Backends(found_backends) => found_backends.run(&config),
            MainSubcommand::CleanCache(backends) => backends.run(&config),
        }
    }
}

impl AddCommand {
    fn run(
        self,
        group_dir: &Path,
        group_files: &BTreeSet<PathBuf>,
        groups: &Groups,
        config: &Config,
    ) -> Result<()> {
        let packages = self.packages.iter().filter(|package| {
            let containing_group_files = groups.contains(self.backend, package);

            if !containing_group_files.is_empty() {
                log::warn!("the {} package for the {} backend is already present in the {containing_group_files:?} group files, so this package has been ignored", package, self.backend);

                false
            } else {
                true
            }
        });

        let group_file = group_dir.join(&self.group).with_extension("toml");

        if config.hostname_groups_enabled && !group_files.contains(&group_file) {
            return Err(eyre!("hostname_groups_enabled is set to true but the group file {}@{group_file:?} is not active for the current hostname, consider choosing one of the active group files: {group_files:?} instead.", &self.group));
        }

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

        doc.entry(&self.backend.to_string().to_lowercase())
            .or_insert(Item::Value(Value::Array(Array::from_iter(
                packages.clone(),
            ))))
            .as_array_mut()
            .wrap_err(eyre!(
                "the {} backend in the {group_file:?} group file has a non-array value",
                self.backend
            ))?
            .extend(packages);

        fs::write(group_file.clone(), doc.to_string())
            .wrap_err(eyre!("writing back modified group file {group_file:?}"))?;

        Ok(())
    }
}

impl RemoveCommand {
    fn run(self, groups: &Groups) -> Result<()> {
        for package in self.packages.iter() {
            let containing_group_files = groups.contains(self.backend, package);
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
                        Value::String(formatted) => package != &formatted.clone().into_value(),
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
        group_files: &BTreeSet<PathBuf>,
        groups: &Groups,
        config: &Config,
    ) -> Result<()> {
        AddCommand {
            backend: self.backend,
            packages: self.packages.clone(),
            group: self.group,
        }
        .run(group_dir, group_files, groups, config)?;

        macro_rules! x {
            ($(($upper_backend:ident, $lower_backend:ident)),*) => {
                match self.backend {
                    $(
                        AnyBackend::$upper_backend => {
                            $upper_backend::install(&self.packages.into_iter().map(|x| (x, Default::default())).collect(), self.no_confirm, config)?;
                        },
                    )*
                }
            };
        }
        apply_public_backends!(x);

        Ok(())
    }
}

impl UninstallCommand {
    fn run(self, groups: &Groups, config: &Config) -> Result<()> {
        RemoveCommand {
            backend: self.backend,
            packages: self.packages.clone(),
        }
        .run(groups)?;

        self.backend.uninstall(
            &self.packages.into_iter().collect(),
            self.no_confirm,
            config,
        )?;

        Ok(())
    }
}

impl CleanCommand {
    fn run(self, required: &Options, config: &Config) -> Result<()> {
        let unmanaged = unmanaged(required, config)?;

        if unmanaged.is_empty() {
            eprintln!("nothing to clean since there are no unmanaged packages");
            return Ok(());
        }

        if self.no_confirm {
            log::info!("proceeding without confirmation");

            unmanaged.uninstall(self.no_confirm, config)
        } else {
            println!("{unmanaged}");

            println!("these packages will be uninstalled\n");

            if Confirm::new()
                .with_prompt("do you want to continue?")
                .default(true)
                .show_default(true)
                .interact()
                .wrap_err("getting user confirmation")?
            {
                unmanaged.uninstall(self.no_confirm, config)
            } else {
                Ok(())
            }
        }
    }
}

impl SyncCommand {
    fn run(self, required: &Options, config: &Config) -> Result<()> {
        let missing = missing(required, config)?;

        if missing.is_empty() {
            log::info!("nothing to do as there are no missing packages");
            return Ok(());
        }

        println!("{}", missing.to_package_ids());

        println!("these packages will be installed\n");

        if self.no_confirm {
            log::info!("proceeding without confirmation");
        } else if !Confirm::new()
            .with_prompt("do you want to continue?")
            .default(true)
            .show_default(true)
            .interact()
            .wrap_err("getting user confirmation")?
        {
            return Ok(());
        }

        missing.install(self.no_confirm, config)
    }
}

impl UnmanagedCommand {
    fn run(self, required: &Options, config: &Config) -> Result<()> {
        let unmanaged = unmanaged(required, config)?;

        if unmanaged.is_empty() {
            eprintln!("no unmanaged packages");
        } else {
            println!("{}", toml::to_string_pretty(&unmanaged)?);
        }

        Ok(())
    }
}

impl BackendsCommand {
    fn run(self, config: &Config) -> Result<()> {
        for backend in AnyBackend::iter() {
            println!(
                "{backend}: {}",
                backend
                    .version(config)
                    .as_deref()
                    .unwrap_or("Not Found")
                    .trim()
            );
        }

        Ok(())
    }
}

impl CleanCacheCommand {
    fn run(&self, config: &Config) -> Result<()> {
        let backends = match &self.backends {
            Some(backends) => {
                let result = backends.iter().map(|x|
                    AnyBackend::from_str(x)
                        .or(Err(eyre!("{x:?} is not a valid backend, run `metapac backends` to see a list of valid backends")))
                ).collect::<Result<Vec<AnyBackend>, _>>();
                result?
            }
            None => AnyBackend::iter().collect(),
        };

        for backend in backends.iter() {
            log::info!("cleaning cache for {backend} backend");

            backend.clean_cache(config)?
        }

        log::info!("cleaned caches of backends: {backends:?}");

        Ok(())
    }
}

fn unmanaged(required: &Options, config: &Config) -> Result<PackageIds> {
    Options::query(config).map(|x| x.to_package_ids().difference(&required.to_package_ids()))
}
fn missing(required: &Options, config: &Config) -> Result<Options> {
    let installed = Options::query(config)?;

    let mut missing = Options::default();

    macro_rules! x {
        ($(($upper_backend:ident, $lower_backend:ident)),*) => {
            $(
                for (package_id, required_options) in required.$lower_backend.iter() {
                    if let Some(missing_options) =
                        $upper_backend::missing(required_options.clone(), installed.$lower_backend.get(package_id).cloned())
                    {
                        missing.$lower_backend.insert(package_id.clone(), missing_options);
                    }
                }
            )*
        };
    }
    apply_public_backends!(x);

    Ok(missing)
}
