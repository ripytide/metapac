use clap::CommandFactory;
use clap_complete::generate;
use std::collections::BTreeSet;
use std::path::Path;
use std::str::FromStr;

use color_eyre::Result;
use color_eyre::eyre::{Context, Ok, eyre};
use dialoguer::Confirm;
use strum::IntoEnumIterator;

use crate::prelude::*;

impl Command {
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

        if config.enabled_backends(&hostname).is_empty() {
            log::warn!("no backends found in the enabled_backends config");
        }

        match self.subcommand {
            MainSubcommand::Update(update) => update.run(&config),
            MainSubcommand::UpdateAll(update_all) => update_all.run(&hostname, &config),
            MainSubcommand::Clean(clean) => clean.run(&hostname, &group_dir, &config),
            MainSubcommand::Sync(sync) => sync.run(&hostname, &group_dir, &config),
            MainSubcommand::Unmanaged(unmanaged) => unmanaged.run(&hostname, &group_dir, &config),
            MainSubcommand::Backends(backends) => backends.run(&config),
            MainSubcommand::CleanCache(clean_cache) => clean_cache.run(&hostname, &config),
            MainSubcommand::Completions(completions) => completions.run(),
        }
    }
}

impl UpdateCommand {
    fn run(self, config: &Config) -> Result<()> {
        let packages = package_vec_to_btreeset(self.packages);

        self.backend
            .update(&packages, self.no_confirm, config.backend_configs())
    }
}

impl UpdateAllCommand {
    fn run(self, hostname: &str, config: &Config) -> Result<()> {
        let enabled_backends = &config.enabled_backends(hostname);
        let backends = parse_backends(&self.backends, enabled_backends)?;

        for backend in backends {
            log::info!("updating all packages for {backend} backend");

            backend.update_all(self.no_confirm, config.backend_configs())?;
        }

        Ok(())
    }
}

impl CleanCommand {
    fn run(self, hostname: &str, group_dir: &Path, config: &Config) -> Result<()> {
        let enabled_backends = config.enabled_backends(hostname);
        let required = required(hostname, group_dir, config)?;
        let installed = installed(&enabled_backends, config.backend_configs())?;
        let unmanaged = unmanaged(&required, &installed)?;

        if unmanaged.is_empty() {
            log::info!("nothing to clean since there are no unmanaged packages");
            return Ok(());
        }

        print!(
            "{}",
            &unmanaged.clone().to_complex().to_raw().to_string_pretty()?
        );

        if self.no_confirm {
            log::info!("proceeding to uninstall packages without confirmation");
        } else if !Confirm::new()
            .with_prompt("these repos/packages will be uninstalled, do you want to continue?")
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
                        $upper_backend::remove_repos(&unmanaged.$lower_backend.repos.keys().cloned().collect(), self.no_confirm, &config.backend_configs().$lower_backend)?;
                        $upper_backend::uninstall_packages(&unmanaged.$lower_backend.packages.keys().cloned().collect(), self.no_confirm, &config.backend_configs().$lower_backend)?;
                    }
                )*
            };
        }
        apply_backends!(x);

        Ok(())
    }
}

impl SyncCommand {
    fn run(self, hostname: &str, group_dir: &Path, config: &Config) -> Result<()> {
        let enabled_backends = config.enabled_backends(hostname);
        let required = required(hostname, group_dir, config)?;
        let missing = missing(&required, &enabled_backends, config.backend_configs())?;

        if missing.is_empty() {
            log::info!("nothing to install as there are no missing packages");
        }

        if !missing.is_empty() {
            print!("{}", &missing.clone().to_raw().to_string_pretty()?);
        }

        if self.no_confirm {
            log::info!("proceeding to install packages without confirmation");
        } else if !missing.is_empty()
            && !Confirm::new()
                .with_prompt("these repos/packages will be installed, do you want to continue?")
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
                        for options in required.$lower_backend.repos.values() {
                            options.hooks.run_before_sync()?;
                        }
                        for options in missing.$lower_backend.repos.values() {
                            options.hooks.run_before_install()?;
                        }
                        $upper_backend::add_repos(&missing.clone().to_non_complex().$lower_backend.repos, self.no_confirm, &config.backend_configs().$lower_backend)?;
                        for options in missing.$lower_backend.repos.values() {
                            options.hooks.run_after_install()?;
                        }
                        for options in required.$lower_backend.repos.values() {
                            options.hooks.run_after_sync()?;
                        }

                        for options in required.$lower_backend.packages.values() {
                            options.hooks.run_before_sync()?;
                        }
                        for options in missing.$lower_backend.packages.values() {
                            options.hooks.run_before_install()?;
                        }
                        $upper_backend::install_packages(&missing.clone().to_non_complex().$lower_backend.packages, self.no_confirm, &config.backend_configs().$lower_backend)?;
                        for options in missing.$lower_backend.packages.values() {
                            options.hooks.run_after_install()?;
                        }
                        for options in required.$lower_backend.packages.values() {
                            options.hooks.run_after_sync()?;
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
    #[allow(clippy::unused_self)]
    fn run(self, hostname: &str, group_dir: &Path, config: &Config) -> Result<()> {
        let enabled_backends = config.enabled_backends(hostname);
        let required = required(hostname, group_dir, config)?;
        let installed = installed(&enabled_backends, config.backend_configs())?;
        let unmanaged = unmanaged(&required, &installed)?;

        if unmanaged.is_empty() {
            log::info!("no unmanaged packages");
        } else {
            print!("{}", &unmanaged.to_complex().to_raw().to_string_pretty()?);
        }

        Ok(())
    }
}

impl BackendsCommand {
    #[allow(clippy::unused_self)]
    fn run(self, config: &Config) -> Result<()> {
        for backend in AnyBackend::iter() {
            println!(
                "{backend}: {}",
                backend
                    .version(config.backend_configs())
                    .as_deref()
                    .unwrap_or("Not Found")
                    .trim()
            );
        }

        Ok(())
    }
}

impl CleanCacheCommand {
    fn run(&self, hostname: &str, config: &Config) -> Result<()> {
        let enabled_backends = &config.enabled_backends(hostname);
        let backends = parse_backends(&self.backends, enabled_backends)?;

        for backend in backends {
            log::info!("cleaning cache for {backend} backend");

            backend.clean_cache(config.backend_configs())?;
        }

        Ok(())
    }
}

impl CompletionsCommand {
    pub fn run(self) -> Result<()> {
        generate(
            self.shell,
            &mut Command::command(),
            "metapac",
            &mut std::io::stdout(),
        );

        Ok(())
    }
}

fn required(hostname: &str, group_dir: &Path, config: &Config) -> Result<AllComplexBackendItems> {
    let enabled_backends = config.enabled_backends(hostname);
    let groups = Groups::load(hostname, group_dir, config)
        .wrap_err("loading package options from group files")?;
    let mut required = groups.to_combined();

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
                let are_valid_packages = $upper_backend::are_packages_valid(&required.clone().$lower_backend.packages.keys().cloned().collect(), &config.backend_configs().$lower_backend);

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

    Ok(required)
}
fn installed(
    enabled_backends: &BTreeSet<AnyBackend>,
    backend_configs: &BackendConfigs,
) -> Result<AllBackendItems> {
    macro_rules! x {
        ($(($upper_backend:ident, $lower_backend:ident)),*) => {
            AllBackendItems {
                $(
                    $lower_backend:
                        if enabled_backends.contains(&AnyBackend::$upper_backend) {
                            BackendItems {
                                packages: $upper_backend::get_installed_packages(&backend_configs.$lower_backend)?,
                                repos: $upper_backend::get_installed_repos(&backend_configs.$lower_backend)?,
                            }
                        } else {
                            Default::default()
                        },
                )*
            }
        };
    }
    Ok(apply_backends!(x))
}
fn unmanaged(
    required: &AllComplexBackendItems,
    installed: &AllBackendItems,
) -> Result<AllBackendItems> {
    let mut unmanaged = AllBackendItems::default();

    macro_rules! x {
        ($(($upper_backend:ident, $lower_backend:ident)),*) => {
            $(
                for (package, options) in installed.$lower_backend.packages.iter() {
                    if (!required.$lower_backend.packages.contains_key(package)) {
                        unmanaged.$lower_backend.packages.insert(package.to_string(), options.clone());
                    }
                }
                for (repo, options) in installed.$lower_backend.repos.iter() {
                    if (!required.$lower_backend.repos.contains_key(repo)) {
                        unmanaged.$lower_backend.repos.insert(repo.to_string(), options.clone());
                    }
                }
            )*
        };
    }
    apply_backends!(x);

    Ok(unmanaged)
}
fn missing(
    required: &AllComplexBackendItems,
    enabled_backends: &BTreeSet<AnyBackend>,
    backend_configs: &BackendConfigs,
) -> Result<AllComplexBackendItems> {
    let installed = installed(enabled_backends, backend_configs)?;

    let mut missing = AllComplexBackendItems::default();

    macro_rules! x {
        ($(($upper_backend:ident, $lower_backend:ident)),*) => {
            $(
                for (package, options) in required.$lower_backend.packages.iter() {
                    if (!installed.$lower_backend.packages.contains_key(package)) {
                        missing.$lower_backend.packages.insert(package.to_string(), options.clone());
                    }
                }
                for (repo, options) in required.$lower_backend.repos.iter() {
                    if (!installed.$lower_backend.repos.contains_key(repo)) {
                        missing.$lower_backend.repos.insert(repo.to_string(), options.clone());
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
