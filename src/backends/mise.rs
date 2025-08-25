use std::collections::{BTreeMap, BTreeSet};

use color_eyre::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Mise;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MiseOptions {}

impl Backend for Mise {
    type Options = MiseOptions;

    fn expand_group_packages(
        packages: BTreeMap<String, Package<Self::Options>>,
        _: &Config,
    ) -> Result<BTreeMap<String, Package<Self::Options>>> {
        Ok(packages)
    }

    fn query(config: &Config) -> Result<BTreeMap<String, Self::Options>> {
        if Self::version(config).is_err() {
            return Ok(BTreeMap::new());
        }

        // Get installed tools with providers and keep only non-delegated providers for mise section
        let tools = list_installed_tools_with_providers(config)?;
        let mut packages = BTreeMap::new();
        for (provider, name) in tools {
            let delegated = match provider.as_str() {
                "npm" => config.mise.manage_backends.contains(&AnyBackend::Npm),
                "pipx" => config.mise.manage_backends.contains(&AnyBackend::Pipx),
                "cargo" => config.mise.manage_backends.contains(&AnyBackend::Cargo),
                _ => false,
            };
            if delegated { continue; }

            let id = if provider == "core" { name.clone() } else { format!("{provider}:{name}") };
            packages.insert(id, Self::Options {});
        }

        Ok(packages)
    }

    fn install(packages: &BTreeMap<String, Self::Options>, _: bool, _: &Config) -> Result<()> {
        for package in packages.keys() {
            run_command(["mise", "install", package], Perms::Same)?;
        }

        Ok(())
    }

    fn uninstall(packages: &BTreeSet<String>, _: bool, _: &Config) -> Result<()> {
        for package in packages {
            run_command(["mise", "uninstall", package], Perms::Same)?;
        }

        Ok(())
    }

    fn update(packages: &BTreeSet<String>, _: bool, _: &Config) -> Result<()> {
        for package in packages {
            run_command(["mise", "upgrade", package], Perms::Same)?;
        }

        Ok(())
    }

    fn update_all(_: bool, _: &Config) -> Result<()> {
        run_command(["mise", "upgrade"], Perms::Same)
    }

    fn clean_cache(_: &Config) -> Result<()> {
        // mise doesn't have a direct cache clean command, so we'll just return Ok
        Ok(())
    }

    fn version(_: &Config) -> Result<String> {
        run_command_for_stdout(["mise", "--version"], Perms::Same, false)
    }
}

/// Returns the installed tools as reported by `mise list --installed`'s first column ("Tool").
/// Examples include:
/// - "node"
/// - "npm:mcp-hub"
/// - "npm:@anthropic-ai/claude-code"
/// - "pipx:ruff"
pub fn query_installed_tools(_: &Config) -> Result<BTreeSet<String>> {
    // Prefer robust JSON output which directly lists tool identifiers as keys.
    // Equivalent CLI: `mise ls --global --json`
    let stdout = run_command_for_stdout(["mise", "ls", "--global", "--json"], Perms::Same, true)?;

    let v: Value = serde_json::from_str(&stdout)?;
    let tools: BTreeSet<String> = if let Some(obj) = v.as_object() {
        obj.keys().cloned().collect()
    } else {
        BTreeSet::new()
    };
    Ok(tools)
}

/// Parse provider/name from a mise tool identifier like "npm:mcp-hub".
/// Returns (provider, name). If no provider is present, returns None.
pub fn parse_provider_and_name(tool_id: &str) -> Option<(String, String)> {
    let mut parts = tool_id.splitn(2, ':');
    let provider = parts.next()?;
    let name = parts.next()?;
    if provider.is_empty() || name.is_empty() {
        return None;
    }
    Some((provider.to_string(), name.to_string()))
}

/// Returns a list of (provider, name) pairs for installed tools.
/// - For keys like "npm:pkg", returns ("npm", "pkg").
/// - For keys without a provider (e.g., "bun"), resolves provider via `mise tool <tool> --backend`.
pub fn list_installed_tools_with_providers(_: &Config) -> Result<Vec<(String, String)>> {
    let stdout = run_command_for_stdout(["mise", "ls", "--global", "--json"], Perms::Same, true)?;

    let v: Value = serde_json::from_str(&stdout)?;
    let mut out: Vec<(String, String)> = Vec::new();
    if let Some(obj) = v.as_object() {
        for key in obj.keys() {
            if let Some((prov, name)) = parse_provider_and_name(key) {
                out.push((prov, name));
            } else {
                // Resolve provider for tools without explicit prefix
                let backend = run_command_for_stdout(["mise", "tool", key.as_str(), "--backend"], Perms::Same, true)?;
                // backend format is like "core:bun" or "npm:cowsay"; trim whitespace/newlines
                let backend = backend.trim();
                if let Some((prov, name)) = parse_provider_and_name(backend) {
                    out.push((prov, name));
                } else {
                    // Fallback: treat as core
                    out.push(("core".to_string(), key.clone()))
                }
            }
        }
    }
    Ok(out)
}

/// Returns the mise provider string for a given backend, if supported for delegation.
pub fn provider_for_backend(backend: &AnyBackend) -> Option<&'static str> {
    match backend {
        AnyBackend::Npm => Some("npm"),
        AnyBackend::Pipx => Some("pipx"),
        AnyBackend::Cargo => Some("cargo"),
        _ => None,
    }
}

/// Whether the given backend is delegated to mise per config.
pub fn is_delegated(config: &Config, backend: &AnyBackend) -> bool {
    config.mise.manage_backends.contains(backend)
}

/// List installed package names for a backend delegated to mise.
pub fn list_names_for_backend(config: &Config, backend: &AnyBackend) -> Result<BTreeSet<String>> {
    let Some(provider) = provider_for_backend(backend) else { return Ok(BTreeSet::new()) };
    let tools = list_installed_tools_with_providers(config)?;
    Ok(tools
        .into_iter()
        .filter_map(|(prov, name)| if prov == provider { Some(name) } else { None })
        .collect())
}

/// Run `mise install provider:pkg` for packages under a delegated backend.
pub fn install_for(backend: &AnyBackend, packages: &BTreeMap<String, String>) -> Result<()> {
    let Some(provider) = provider_for_backend(backend) else { return Ok(()) };
    if packages.is_empty() { return Ok(()); }
    let mut args: Vec<String> = vec!["mise".into(), "install".into()];
    args.extend(packages.keys().map(|k| format!("{provider}:{k}")));
    run_command(args, Perms::Same)
}

/// Run `mise uninstall provider:pkg` for packages under a delegated backend.
pub fn uninstall_for(backend: &AnyBackend, packages: &BTreeSet<String>) -> Result<()> {
    let Some(provider) = provider_for_backend(backend) else { return Ok(()) };
    for package in packages {
        run_command(["mise", "uninstall", &format!("{provider}:{package}")], Perms::Same)?;
    }
    Ok(())
}

/// Run `mise upgrade provider:pkg` for the given packages.
pub fn upgrade_for(backend: &AnyBackend, packages: &BTreeSet<String>) -> Result<()> {
    let Some(provider) = provider_for_backend(backend) else { return Ok(()) };
    for package in packages {
        run_command(["mise", "upgrade", &format!("{provider}:{package}")], Perms::Same)?;
    }
    Ok(())
}

/// Run `mise upgrade provider:*` for upgrade-all.
pub fn upgrade_all_for(backend: &AnyBackend) -> Result<()> {
    let Some(provider) = provider_for_backend(backend) else { return Ok(()) };
    run_command(["mise", "upgrade", &format!("{provider}:*")], Perms::Same)
}
