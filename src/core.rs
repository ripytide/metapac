use std::fs::{self, read_to_string, File};
use std::path::Path;

use color_eyre::eyre::{eyre, Context, ContextCompat};
use color_eyre::Result;
use dialoguer::Confirm;
use toml_edit::{Array, DocumentMut, Item, Value};

use crate::backends::all::backend_versions;
use crate::cli::BackendsCommand;
use crate::prelude::*;
use crate::review::review;

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
        let groups = Groups::load(&group_dir, &hostname, &config)
            .wrap_err("failed to load package install options from groups")?;

        let managed = groups.to_install_options().map_install_packages(&config)?;

        match self.subcommand {
            MainSubcommand::Clean(clean) => clean.run(&managed, &config),
            MainSubcommand::Add(add) => add.run(&group_dir, &groups),
            MainSubcommand::Review(review) => review.run(&managed, &config),
            MainSubcommand::Sync(sync) => sync.run(&managed, &config),
            MainSubcommand::Unmanaged(unmanaged) => unmanaged.run(&managed, &config),
            MainSubcommand::Backends(found_backends) => found_backends.run(&config),
        }
    }
}

impl CleanCommand {
    fn run(self, managed: &InstallOptions, config: &Config) -> Result<()> {
        let unmanaged = unmanaged(managed, config)?;

        if unmanaged.is_empty() {
            eprintln!("nothing to clean since there are no unmanaged packages");
            return Ok(());
        }

        if self.no_confirm {
            log::info!("proceeding without confirmation");

            unmanaged.remove_packages(self.no_confirm, config)
        } else {
            println!("{unmanaged}");

            println!("these packages will be removed\n");

            if Confirm::new()
                .with_prompt("do you want to continue?")
                .default(true)
                .show_default(true)
                .interact()
                .wrap_err("getting user confirmation")?
            {
                unmanaged.remove_packages(self.no_confirm, config)
            } else {
                Ok(())
            }
        }
    }
}

impl AddCommand {
    fn run(self, group_dir: &Path, groups: &Groups) -> Result<()> {
        let containing_group_files = groups.contains(self.backend, &self.package);
        if !containing_group_files.is_empty() {
            log::info!("the {} package for the {} backend is already installed in the {containing_group_files:?} group files", self.package, self.backend);
        }

        let group_file = group_dir.join(&self.group).with_extension("toml");

        log::info!("parsing group file: {}@{group_file:?}", &self.group);

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
            .or_insert(Item::Value(Value::Array(Array::from_iter([self
                .package
                .clone()]))))
            .as_array_mut()
            .wrap_err(eyre!(
                "the {} backend in the {group_file:?} group file has a non-array value",
                self.backend
            ))?
            .push(self.package);

        fs::write(group_file, doc.to_string())
            .wrap_err("writing back modified group file {group_file:?}")?;

        Ok(())
    }
}

impl ReviewCommand {
    fn run(self, _: &InstallOptions, _: &Config) -> Result<()> {
        review()
    }
}

impl SyncCommand {
    fn run(self, managed: &InstallOptions, config: &Config) -> Result<()> {
        let missing = missing(managed, config)?;

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

        missing.install_packages(self.no_confirm, config)
    }
}

impl UnmanagedCommand {
    fn run(self, managed: &InstallOptions, config: &Config) -> Result<()> {
        let unmanaged = unmanaged(managed, config)?;

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
        for (backend, version) in backend_versions(config) {
            println!("{backend}:");
            println!("{version}");
        }
        Ok(())
    }
}

fn unmanaged(managed: &InstallOptions, config: &Config) -> Result<PackageIds> {
    QueryInfos::query_installed_packages(config)
        .map(|x| x.to_package_ids().difference(&managed.to_package_ids()))
}
fn missing(managed: &InstallOptions, config: &Config) -> Result<InstallOptions> {
    let installed = QueryInfos::query_installed_packages(config)?.to_package_ids();

    let mut missing = managed.clone();

    macro_rules! x {
        ($(($upper_backend:ident, $lower_backend:ident)),*) => {
            $(
                for package_id in installed.$lower_backend {
                    missing.$lower_backend.remove(&package_id);
                }
            )*
        };
    }
    apply_public_backends!(x);

    Ok(missing)
}
