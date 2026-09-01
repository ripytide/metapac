use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;

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
    #[serde(default)]
    pub gpgkey: Option<String>,
}

fn is_user_installed(line: &str) -> bool {
    matches!(line.split('|').next().map(str::trim), Some("i+" | "il"))
}

fn parse_repos(stdout: &str) -> Result<BTreeMap<String, ZypperRepoOptions>> {
    let mut repos = BTreeMap::new();
    let mut alias = None;
    let mut url = None;
    let mut gpgkey = None;

    let add_repo = |repos: &mut BTreeMap<_, _>,
                    alias: &mut Option<String>,
                    url: &mut Option<String>,
                    gpgkey: &mut Option<String>|
     -> Result<()> {
        if let Some(alias) = alias.take() {
            let url = url.take().ok_or(eyre!("unexpected zypper repo output"))?;
            repos.insert(
                alias,
                ZypperRepoOptions {
                    url,
                    gpgkey: gpgkey.take(),
                },
            );
        }
        Ok(())
    };

    for line in stdout.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(section) = line
            .strip_prefix('[')
            .and_then(|line| line.strip_suffix(']'))
        {
            add_repo(&mut repos, &mut alias, &mut url, &mut gpgkey)?;
            if section.is_empty() {
                return Err(eyre!("unexpected zypper repo output"));
            }
            alias = Some(section.to_string());
            continue;
        }

        let (key, value) = line
            .split_once('=')
            .ok_or(eyre!("unexpected zypper repo output"))?;
        match key.trim() {
            "baseurl" => url = Some(value.trim().to_string()),
            "gpgkey" => gpgkey = Some(value.trim().to_string()),
            _ => {}
        }
    }

    add_repo(&mut repos, &mut alias, &mut url, &mut gpgkey)?;
    Ok(repos)
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

        let stdout = run_command_for_stdout(
            ["zypper", "packages", "--userinstalled"],
            Perms::Same,
            StdErr::Show,
        )?;

        stdout
            .lines()
            .filter(|line| is_user_installed(line))
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
        let repos = run_command_for_stdout(
            ["zypper", "repos", "--show-enabled-only", "--export", "-"],
            Perms::Same,
            StdErr::Show,
        )?;

        Ok(parse_repos(&repos)?
            .into_iter()
            .filter(|(alias, _)| !alias.starts_with("openSUSE:"))
            .collect())
    }

    fn add_repos(
        repos: &BTreeMap<String, Self::RepoOptions>,
        no_confirm: bool,
        _: &Self::Config,
    ) -> Result<()> {
        for (alias, options) in repos {
            if options.url.is_empty() {
                return Err(eyre!("zypper repo {alias:?} has an empty url"));
            }

            // `zypper addrepo` accepts either a URL plus alias or a `.repo` file, but not
            // individual settings like `gpgkey`. When a gpgkey is present, write a temporary
            // `.repo` definition so Zypper can add the repository in one step.
            let repo_file = options
                .gpgkey
                .as_ref()
                .map(|gpgkey| {
                    let mut file = tempfile::NamedTempFile::new()?;
                    writeln!(file, "[{alias}]")?;
                    writeln!(file, "name={alias}")?;
                    writeln!(file, "enabled=1")?;
                    writeln!(file, "autorefresh=1")?;
                    writeln!(file, "baseurl={}", options.url)?;
                    writeln!(file, "type=rpm-md")?;
                    writeln!(file, "gpgcheck=1")?;
                    writeln!(file, "gpgkey={gpgkey}")?;
                    file.flush()?;
                    Ok::<_, color_eyre::eyre::Report>(file)
                })
                .transpose()?;

            let url = if let Some(file) = repo_file.as_ref() {
                file.path()
                    .to_str()
                    .ok_or(eyre!("temporary zypper repo file path is not valid UTF-8"))?
            } else {
                options.url.as_str()
            };

            let repo_file_url = repo_file.is_some()
                || std::path::Path::new(&options.url)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("repo"));
            let mut args = vec!["zypper"];
            if no_confirm {
                args.push("--non-interactive");
            }
            if repo_file_url {
                args.extend(["addrepo", "--repo"]);
                args.push(url);
            } else {
                args.extend(["addrepo", "--refresh"]);
                args.extend([url, alias.as_str()]);
            }

            run_command(args, Perms::Sudo)?;
        }

        Ok(())
    }

    fn remove_repos(repos: &BTreeSet<String>, no_confirm: bool, _: &Self::Config) -> Result<()> {
        for alias in repos {
            run_command(
                ["zypper"]
                    .into_iter()
                    .chain(no_confirm.then_some("--non-interactive"))
                    .chain(["removerepo", alias.as_str()]),
                Perms::Sudo,
            )?;
        }

        Ok(())
    }

    fn version(_: &Self::Config) -> Result<String> {
        run_command_for_stdout(["zypper", "--version"], Perms::Same, StdErr::Show)
    }
}

#[cfg(test)]
mod tests {
    use super::{is_user_installed, parse_repos};

    #[test]
    fn parses_exported_repo_output() {
        let repos = parse_repos(
            "[packman]\n\
             name=Packman\n\
             enabled=1\n\
             baseurl=https://ftp.gwdg.de/pub/linux/misc/packman/suse/$releasever/\n\
             gpgcheck=1\n\
             \n\
             [brave-browser]\n\
             name=Brave Browser\n\
             enabled=1\n\
             baseurl=https://brave-browser-rpm-release.s3.brave.com/x86_64\n\
             gpgcheck=1\n\
             gpgkey=https://brave-browser-rpm-release.s3.brave.com/brave-core.asc\n",
        )
        .unwrap();

        assert_eq!(
            repos["packman"].url,
            "https://ftp.gwdg.de/pub/linux/misc/packman/suse/$releasever/"
        );
        assert_eq!(
            repos["brave-browser"].url,
            "https://brave-browser-rpm-release.s3.brave.com/x86_64"
        );
        assert_eq!(
            repos["brave-browser"].gpgkey.as_deref(),
            Some("https://brave-browser-rpm-release.s3.brave.com/brave-core.asc")
        );
    }

    #[test]
    fn rejects_invalid_exported_repo_output() {
        assert!(parse_repos("[packman]\nname=Packman\n").is_err());
    }

    #[test]
    fn recognizes_locked_userinstalled_packages() {
        let packages = "il | @System | kernel-default | 7.1.5-1.1 | x86_64\n\
                        vl | repo-oss | kernel-default | 7.1.6-1.1 | x86_64\n";

        let installed = packages
            .lines()
            .filter(|line| is_user_installed(line))
            .map(|line| {
                line.split('|')
                    .nth(2)
                    .expect("package column")
                    .trim()
                    .to_string()
            })
            .collect::<Vec<_>>();

        assert_eq!(installed, ["kernel-default"]);
    }
}
