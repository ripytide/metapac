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

#[serde_inline_default]
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RustupOptions {
    #[serde_inline_default(RustupOptions::default().components)]
    pub components: BTreeSet<String>,
}

impl Backend for Rustup {
    type Options = RustupOptions;

    fn map_required(
        packages: BTreeMap<String, Self::Options>,
        _: &Config,
    ) -> Result<BTreeMap<String, Self::Options>> {
        Ok(packages)
    }

    fn query(config: &Config) -> Result<BTreeMap<String, Self::Options>> {
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
                    Self::Options {
                        components: components_stdout.lines().map(|x| x.to_string()).collect(),
                    },
                );
            }
        }

        Ok(packages)
    }

    fn install(
        packages: &BTreeMap<String, Self::Options>,
        _: bool,
        _: &Config,
    ) -> Result<()> {
        for (toolchain, rustup_options) in packages.iter() {
            run_command(
                ["rustup", "toolchain", "install", toolchain.as_str()],
                Perms::Same,
            )?;

            if !rustup_options.components.is_empty() {
                run_command(
                    [
                        "rustup",
                        "component",
                        "add",
                        "--toolchain",
                        toolchain.as_str(),
                    ]
                    .into_iter()
                    .chain(rustup_options.components.iter().map(String::as_str)),
                    Perms::Same,
                )?;
            }
        }

        Ok(())
    }

    fn remove(packages: &BTreeSet<String>, _: bool, _: &Config) -> Result<()> {
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

    fn missing(required: Self::Options, installed: Option<Self::Options>) -> Option<Self::Options> {
        match installed {
            Some(installed) => {
                let missing = required
                    .components
                    .difference(&installed.components)
                    .cloned()
                    .collect::<BTreeSet<_>>();
                if missing.is_empty() {
                    None
                } else {
                    Some(Self::Options {
                        components: missing,
                    })
                }
            }
            None => Some(required),
        }
    }
}
