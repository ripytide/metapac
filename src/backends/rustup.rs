use crate::cmd::run_command;
use crate::cmd::run_command_for_stdout;
use crate::prelude::*;
use color_eyre::Result;
use serde::Deserialize;
use serde::Serialize;
use serde_inline_default::serde_inline_default;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Rustup;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RustupQueryInfo {
    pub components: BTreeSet<String>,
}

#[serde_inline_default]
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RustupInstallOptions {
    #[serde_inline_default(RustupInstallOptions::default().components)]
    pub components: BTreeSet<String>,
}

impl Backend for Rustup {
    type QueryInfo = RustupQueryInfo;
    type InstallOptions = RustupInstallOptions;

    fn map_managed_packages(
        packages: BTreeMap<String, Self::InstallOptions>,
        _: &Config,
    ) -> Result<BTreeMap<String, Self::InstallOptions>> {
        Ok(packages)
    }

    fn query_installed_packages(config: &Config) -> Result<BTreeMap<String, Self::QueryInfo>> {
        if Self::version(config).is_err() {
            return Ok(BTreeMap::new());
        }

        let mut packages = BTreeMap::new();

        let toolchains_stdout =
            run_command_for_stdout(["rustup", "toolchain", "list"], Perms::Same, false)?;
        let toolchains = toolchains_stdout.lines().map(|x| {
            x.split(' ')
                .next()
                .expect("output shouldn't contain empty lines")
                .to_string()
        });

        for toolchain in toolchains {
            //due to https://github.com/rust-lang/rustup/issues/1570
            //we sometimes must interpret a failed command as no
            //components for custom toolchains
            if let Ok(components_stdout) = run_command_for_stdout(
                [
                    "rustup",
                    "component",
                    "list",
                    "--installed",
                    "--toolchain",
                    toolchain.as_str(),
                ],
                Perms::Same,
                false,
            ) {
                packages.insert(
                    toolchain,
                    RustupQueryInfo {
                        components: components_stdout.lines().map(|x| x.to_string()).collect(),
                    },
                );
            }
        }

        Ok(packages)
    }

    fn install_packages(
        packages: &BTreeMap<String, Self::InstallOptions>,
        _: bool,
        _: &Config,
    ) -> Result<()> {
        for (toolchain, rustup_install_options) in packages.iter() {
            run_command(
                ["rustup", "toolchain", "install", toolchain.as_str()],
                Perms::Same,
            )?;

            if !rustup_install_options.components.is_empty() {
                run_command(
                    [
                        "rustup",
                        "component",
                        "add",
                        "--toolchain",
                        toolchain.as_str(),
                    ]
                    .into_iter()
                    .chain(rustup_install_options.components.iter().map(String::as_str)),
                    Perms::Same,
                )?;
            }
        }

        Ok(())
    }

    fn remove_packages(packages: &BTreeSet<String>, _: bool, _: &Config) -> Result<()> {
        if !packages.is_empty() {
            for toolchain in packages.iter() {
                run_command(
                    ["rustup", "toolchain", "remove", toolchain.as_str()],
                    Perms::Same,
                )?;
            }
        }

        Ok(())
    }

    fn clean_cache(_: &Config) -> Result<()> {
        Ok(())
    }

    fn version(_: &Config) -> Result<String> {
        run_command_for_stdout(["rustup", "--version"], Perms::Same, true)
    }

    fn missing(
        managed: Self::InstallOptions,
        installed: Option<Self::QueryInfo>,
    ) -> Option<Self::InstallOptions> {
        match installed {
            Some(installed) => {
                let missing = managed
                    .components
                    .difference(&installed.components)
                    .cloned()
                    .collect::<BTreeSet<_>>();
                if missing.is_empty() {
                    None
                } else {
                    Some(RustupInstallOptions {
                        components: missing,
                    })
                }
            }
            None => Some(managed),
        }
    }
}
