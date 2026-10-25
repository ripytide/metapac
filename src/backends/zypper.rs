use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Zypper;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ZypperConfig {
    #[serde(default)]
    pub distribution_upgrade: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZypperPackageOptions {}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZypperRepoOptions {
    pub url: String,
}

impl Backend for Zypper {
    type Config = ZypperConfig;
    type PackageOptions = ZypperPackageOptions;
    type RepoOptions = ZypperRepoOptions;

    fn invalid_package_help_text() -> String {
        String::new()
    }

    fn is_valid_package_name(_: &str) -> Option<bool> {
        None
    }

    fn get_all_packages(_: &Self::Config) -> Result<BTreeSet<String>> {
        Err(eyre!("unimplemented"))
    }

    fn get_installed_packages(
        config: &Self::Config,
    ) -> Result<std::collections::BTreeMap<String, Self::PackageOptions>> {
        if Self::version(config).is_err() {
            return Ok(BTreeMap::new());
        }

        // only read the installed packages so that zypper doesn't refresh any repos, as that
        // could ask about a new signing key while the output is hidden
        let stdout = run_command_for_stdout(
            [
                "zypper",
                "--disable-repositories",
                "packages",
                "--userinstalled",
            ],
            Perms::Same,
            StdErr::Show,
        )?;

        stdout
            .lines()
            // locked packages show "il" instead of "i+"
            .filter(|line| matches!(line.split('|').next().map(str::trim), Some("i+" | "il")))
            .map(|line| -> Result<(String, Self::PackageOptions)> {
                let mut parts = line.split('|');
                let package = parts
                    .nth(2)
                    .ok_or(eyre!("unexpected output"))?
                    .trim()
                    .to_string();
                Ok((package, Self::PackageOptions {}))
            })
            .collect()
    }

    fn install_packages(
        packages: &BTreeMap<String, Self::PackageOptions>,
        no_confirm: bool,
        _: &Self::Config,
    ) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["zypper", "install"]
                    .into_iter()
                    .chain(no_confirm.then_some("--no-confirm"))
                    .chain(packages.keys().map(String::as_str)),
                Perms::Sudo,
            )?;
        }

        Ok(())
    }

    fn uninstall_packages(
        packages: &BTreeSet<String>,
        no_confirm: bool,
        _: &Self::Config,
    ) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["zypper", "remove"]
                    .into_iter()
                    .chain(no_confirm.then_some("--no-confirm"))
                    .chain(packages.iter().map(String::as_str)),
                Perms::Sudo,
            )?;
        }

        Ok(())
    }

    fn update_packages(
        packages: &BTreeSet<String>,
        no_confirm: bool,
        _: &Self::Config,
    ) -> Result<()> {
        if !packages.is_empty() {
            run_command(
                ["zypper", "update"]
                    .into_iter()
                    .chain(no_confirm.then_some("--no-confirm"))
                    .chain(packages.iter().map(String::as_str)),
                Perms::Sudo,
            )?;
        }

        Ok(())
    }

    fn update_all_packages(no_confirm: bool, config: &Self::Config) -> Result<()> {
        run_command(
            [
                "zypper",
                if config.distribution_upgrade {
                    "dist-upgrade"
                } else {
                    "update"
                },
            ]
            .into_iter()
            .chain(no_confirm.then_some("--no-confirm")),
            Perms::Sudo,
        )
    }

    fn clean_cache(config: &Self::Config) -> Result<()> {
        Self::version(config).map_or(Ok(()), |_| run_command(["zypper", "clean"], Perms::Sudo))
    }

    fn refresh(_: &Self::Config) -> Result<()> {
        run_command(["zypper", "refresh"], Perms::Sudo)
    }

    fn get_installed_repos(_: &Self::Config) -> Result<BTreeMap<String, Self::RepoOptions>> {
        let output = run_command_for_stdout(
            ["zypper", "repos", "--export", "-"],
            Perms::Same,
            StdErr::Show,
        )?;

        // example output:
        // [vscode]
        // name=Visual Studio Code
        // enabled=1
        // baseurl=https://packages.microsoft.com/yumrepos/vscode
        // gpgcheck=1

        let mut repos = BTreeMap::new();
        let mut repo = None;
        for line in output.lines().map(str::trim) {
            if let Some(header) = line.strip_prefix('[').and_then(|x| x.strip_suffix(']')) {
                repo = Some(header);
            } else if let Some(url) = line.strip_prefix("baseurl=") {
                repos.insert(
                    repo.ok_or(eyre!("unexpected output"))?.to_string(),
                    Self::RepoOptions {
                        url: url.to_string(),
                    },
                );
            }
        }

        // the default distribution repos are managed by the openSUSE service
        repos.retain(|repo: &String, _| !repo.starts_with("openSUSE:"));

        Ok(repos)
    }

    fn add_repos(
        repos: &BTreeMap<String, Self::RepoOptions>,
        no_confirm: bool,
        _: &Self::Config,
    ) -> Result<()> {
        for (repo, options) in repos {
            // .repo files contain their own alias along with any other settings such as gpgkey
            let args = if std::path::Path::new(&options.url)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("repo"))
            {
                vec!["--repo", options.url.as_str()]
            } else {
                vec![options.url.as_str(), repo.as_str()]
            };

            run_command(
                ["zypper"]
                    .into_iter()
                    .chain(no_confirm.then_some("--non-interactive"))
                    .chain(["addrepo", "--refresh"])
                    .chain(args),
                Perms::Sudo,
            )?;

            // refresh now so that zypper asks about any new signing key while the repo is
            // being added
            run_command(
                ["zypper"]
                    .into_iter()
                    .chain(no_confirm.then_some("--non-interactive"))
                    .chain(no_confirm.then_some("--gpg-auto-import-keys"))
                    .chain(["refresh", repo.as_str()]),
                Perms::Sudo,
            )?;
        }

        Ok(())
    }

    fn remove_repos(repos: &BTreeSet<String>, no_confirm: bool, _: &Self::Config) -> Result<()> {
        for repo in repos {
            run_command(
                ["zypper"]
                    .into_iter()
                    .chain(no_confirm.then_some("--non-interactive"))
                    .chain(["removerepo", repo.as_str()]),
                Perms::Sudo,
            )?;
        }

        Ok(())
    }

    fn version(_: &Self::Config) -> Result<String> {
        run_command_for_stdout(["zypper", "--version"], Perms::Same, StdErr::Show)
    }
}
