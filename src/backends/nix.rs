use std::collections::{BTreeMap, BTreeSet};

use color_eyre::Result;
use color_eyre::eyre::eyre;
use indoc::formatdoc;
use serde::{Deserialize, Serialize};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Nix;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct NixConfig {
    #[serde(default)]
    pub profile: Option<String>,
    #[serde(default)]
    pub impure: bool,
    #[serde(default)]
    pub accept_flake_config: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NixPackageOptions {
    pub installable: Option<String>,
    pub priority: Option<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NixRepoOptions {}

#[derive(Debug, Deserialize)]
struct NixProfileList {
    elements: BTreeMap<String, NixProfileElement>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NixProfileElement {
    attr_path: Option<String>,
    original_url: Option<String>,
    priority: Option<u32>,
}

impl Backend for Nix {
    type Config = NixConfig;
    type PackageOptions = NixPackageOptions;
    type RepoOptions = NixRepoOptions;

    fn invalid_package_help_text() -> String {
        formatdoc! {"
            A nix package may be invalid due to one of the following issues:
                - package names must not be empty, contain whitespace, or start with '-'
                - package names are matched against nix profile element names, so if a package
                  keeps showing as unmanaged run `metapac unmanaged` and copy the generated name
                - if the package source is not `nixpkgs#<name>`, set `options.installable`
                  explicitly to the installable you want metapac to install
        "}
    }

    fn is_valid_package_name(package: &str) -> Option<bool> {
        let has_whitespace = package.chars().any(char::is_whitespace);

        if package.trim().is_empty() || package.starts_with('-') || has_whitespace {
            Some(false)
        } else {
            None
        }
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

        let mut args = vec![
            "nix".to_string(),
            "profile".to_string(),
            "list".to_string(),
            "--json".to_string(),
            "--no-pretty".to_string(),
        ];
        append_profile_arg(&mut args, config);

        let output = run_command_for_stdout(args, Perms::Same, StdErr::Show)?;
        parse_installed_packages(&output)
    }

    fn install_packages(
        packages: &BTreeMap<String, Self::PackageOptions>,
        _: bool,
        config: &Self::Config,
    ) -> Result<()> {
        for (name, options) in packages {
            let mut args = vec!["nix".to_string(), "profile".to_string(), "add".to_string()];
            append_profile_arg(&mut args, config);
            append_eval_flags(&mut args, config);

            if let Some(priority) = options.priority {
                args.push("--priority".to_string());
                args.push(priority.to_string());
            }

            args.push(installable_for(name, options));

            run_command(args, Perms::Same)?;
        }

        Ok(())
    }

    fn uninstall_packages(
        packages: &BTreeSet<String>,
        _: bool,
        config: &Self::Config,
    ) -> Result<()> {
        if !packages.is_empty() {
            let mut args = vec![
                "nix".to_string(),
                "profile".to_string(),
                "remove".to_string(),
            ];
            append_profile_arg(&mut args, config);
            args.extend(packages.iter().cloned());
            run_command(args, Perms::Same)?;
        }

        Ok(())
    }

    fn update_packages(packages: &BTreeSet<String>, _: bool, config: &Self::Config) -> Result<()> {
        if !packages.is_empty() {
            let mut args = vec![
                "nix".to_string(),
                "profile".to_string(),
                "upgrade".to_string(),
            ];
            append_profile_arg(&mut args, config);
            append_eval_flags(&mut args, config);
            args.extend(packages.iter().cloned());
            run_command(args, Perms::Same)?;
        }

        Ok(())
    }

    fn update_all_packages(_: bool, config: &Self::Config) -> Result<()> {
        let mut args = vec![
            "nix".to_string(),
            "profile".to_string(),
            "upgrade".to_string(),
            "--all".to_string(),
        ];
        append_profile_arg(&mut args, config);
        append_eval_flags(&mut args, config);
        run_command(args, Perms::Same)
    }

    fn clean_cache(config: &Self::Config) -> Result<()> {
        Self::version(config).map_or(Ok(()), |_| run_command(["nix", "store", "gc"], Perms::Same))
    }

    fn get_installed_repos(_: &Self::Config) -> Result<BTreeMap<String, Self::RepoOptions>> {
        Ok(BTreeMap::new())
    }

    fn add_repos(
        repos: &BTreeMap<String, Self::RepoOptions>,
        _: bool,
        _: &Self::Config,
    ) -> Result<()> {
        if repos.is_empty() {
            Ok(())
        } else {
            Err(eyre!("unimplemented"))
        }
    }

    fn remove_repos(repos: &BTreeSet<String>, _: bool, _: &Self::Config) -> Result<()> {
        if repos.is_empty() {
            Ok(())
        } else {
            Err(eyre!("unimplemented"))
        }
    }

    fn version(_: &Self::Config) -> Result<String> {
        run_command_for_stdout(["nix", "--version"], Perms::Same, StdErr::Show)
    }
}

fn append_profile_arg(args: &mut Vec<String>, config: &NixConfig) {
    if let Some(profile) = &config.profile {
        args.push("--profile".to_string());
        args.push(profile.clone());
    }
}

fn append_eval_flags(args: &mut Vec<String>, config: &NixConfig) {
    if config.impure {
        args.push("--impure".to_string());
    }
    if config.accept_flake_config {
        args.push("--accept-flake-config".to_string());
    }
}

fn installable_for(name: &str, options: &NixPackageOptions) -> String {
    options
        .installable
        .clone()
        .unwrap_or_else(|| format!("nixpkgs#{name}"))
}

fn parse_installed_packages(stdout: &str) -> Result<BTreeMap<String, NixPackageOptions>> {
    let profile_list: NixProfileList = serde_json::from_str(stdout)?;

    let installed = profile_list
        .elements
        .into_iter()
        .map(|(name, element)| {
            let installable = element
                .original_url
                .zip(element.attr_path)
                .map(|(original_url, attr_path)| format!("{original_url}#{attr_path}"));

            let priority = element.priority.filter(|priority| *priority != 5);

            (
                name,
                NixPackageOptions {
                    installable,
                    priority,
                },
            )
        })
        .collect();

    Ok(installed)
}
