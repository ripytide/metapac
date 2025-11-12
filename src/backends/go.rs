use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use color_eyre::Result;
use color_eyre::eyre::{Context, eyre};
use serde::{Deserialize, Serialize};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Go;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoOptions {
    #[serde(default)]
    version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct GoConfig {
    #[serde(default)]
    enable_updates: bool,
}

impl Backend for Go {
    type Options = GoOptions;
    type Config = GoConfig;

    fn invalid_package_help_text() -> String {
        String::new()
    }

    fn is_valid_package_name(_: &str) -> Option<bool> {
        None
    }

    fn get_all(_: &Self::Config) -> Result<BTreeSet<String>> {
        Err(eyre!("unimplemented"))
    }

    fn get_installed(_: &Self::Config) -> Result<BTreeMap<String, Self::Options>> {
        if Self::version(&GoConfig::default()).is_err() {
            return Ok(BTreeMap::new());
        }

        let bin_dir = get_gobin()?;

        if !bin_dir.exists() {
            return Ok(BTreeMap::new());
        }

        let mut packages = BTreeMap::new();

        for entry in std::fs::read_dir(&bin_dir).wrap_err("reading go bin directory")? {
            let entry = entry.wrap_err("reading directory entry")?;
            let path = entry.path();

            if path.is_file()
                && is_executable(&path)
                && let Some(binary_name) = path.file_name().and_then(|n| n.to_str())
            {
                // Go doesn't track which import path created which binary,
                // so we store by binary name. Users should use import paths in
                // group files, and we'll extract the binary name for matching.
                packages.insert(binary_name.to_string(), GoOptions { version: None });
            }
        }

        Ok(packages)
    }

    fn install(
        packages: &BTreeMap<String, Self::Options>,
        _: bool,
        _: &Self::Config,
    ) -> Result<()> {
        for (package, options) in packages {
            let version_spec = if let Some(version) = &options.version {
                format!("@{version}")
            } else {
                "@latest".to_string()
            };

            run_command(
                ["go", "install", &format!("{package}{version_spec}")],
                Perms::Same,
            )?;
        }

        Ok(())
    }

    fn uninstall(packages: &BTreeSet<String>, _: bool, _: &Self::Config) -> Result<()> {
        let bin_dir = get_gobin()?;

        if !bin_dir.exists() {
            return Ok(());
        }

        for package in packages {
            // Extract binary name from import path (last component)
            let binary_name = extract_binary_name(package);
            let binary_path = bin_dir.join(&binary_name);
            if binary_path.exists() {
                std::fs::remove_file(&binary_path)
                    .wrap_err(format!("removing binary: {}", binary_path.display()))?;
            }
        }

        Ok(())
    }

    fn update(packages: &BTreeSet<String>, no_confirm: bool, config: &Self::Config) -> Result<()> {
        if config.enable_updates {
            Self::install(
                &BTreeMap::from_iter(packages.iter().map(|p| {
                    (
                        p.clone(),
                        GoOptions {
                            ..Default::default()
                        },
                    )
                })),
                no_confirm,
                config,
            )
        } else {
            Ok(())
        }
    }

    fn update_all(no_confirm: bool, config: &Self::Config) -> Result<()> {
        // As we can't get package versions, ask for behaviour in config
        if config.enable_updates {
            Self::install(&Self::get_installed(config)?, no_confirm, config)
        } else {
            Ok(())
        }
    }

    fn clean_cache(_: &Self::Config) -> Result<()> {
        run_command(["go", "clean", "-modcache"], Perms::Same)
    }

    fn version(_: &Self::Config) -> Result<String> {
        run_command_for_stdout(["go", "version"], Perms::Same, false)
    }
}

fn get_gobin() -> Result<PathBuf> {
    std::env::var("GOBIN").map(PathBuf::from).or_else(|_| {
        let gopath = std::env::var("GOPATH")
            .or_else(|_| {
                // If GOPATH is not set, default to $HOME/go
                home::home_dir()
                    .map(|p| p.join("go").to_string_lossy().to_string())
                    .ok_or(std::env::VarError::NotPresent)
            })
            .wrap_err("getting GOPATH")?;

        Ok(PathBuf::from(gopath).join("bin"))
    })
}

/// Check if a file is an executable binary.
/// On Unix systems, checks if the file has executable permissions.
/// On Windows, checks if the file has a .exe extension.
fn is_executable(path: &std::path::Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path)
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }

    #[cfg(windows)]
    {
        // On Windows, check for .exe extension
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("exe"))
            .unwrap_or(false)
    }

    #[cfg(not(any(unix, windows)))]
    {
        // For other platforms, just check if it's a file
        path.is_file()
    }
}

/// Extract binary name from Go import path.
/// For example: "github.com/user/repo/cmd/tool" -> "tool"
fn extract_binary_name(import_path: &str) -> String {
    import_path
        .split('/')
        .next_back()
        .unwrap_or(import_path)
        .to_string()
}
