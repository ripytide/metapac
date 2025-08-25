use std::collections::{BTreeMap, BTreeSet};

use color_eyre::Result;
use serde::{Deserialize, Serialize};

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

        // Get installed tools and filter out provider-prefixed tools that are delegated
        // to their native backends via [mise].manage_backends. This keeps mise's section
        // for core tools (bun, deno, node, python, etc.) and non-delegated providers.
        let tools = query_installed_tools(config)?;
        let mut packages = BTreeMap::new();

        for tool in tools {
            if let Some((provider, _name)) = parse_provider_and_name(&tool) {
                let delegated = match provider.as_str() {
                    "npm" => config.mise.manage_backends.contains(&AnyBackend::Npm),
                    "pipx" => config.mise.manage_backends.contains(&AnyBackend::Pipx),
                    "cargo" => config.mise.manage_backends.contains(&AnyBackend::Cargo),
                    _ => false,
                };
                if delegated {
                    continue;
                }
            }
            packages.insert(tool, Self::Options {});
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
    // We intentionally do not depend on the registry to resolve aliases here; we only care
    // about the tool identifier as shown by mise to keep behavior stable and predictable.
    let output = run_command_for_stdout(["mise", "list", "--installed"], Perms::Same, true)?;

    let mut tools: BTreeSet<String> = BTreeSet::new();
    for (i, line) in output.lines().enumerate() {
        // Skip header line if present (starts with "Tool"), and any empty lines
        let first = line.split_whitespace().next();
        let Some(first) = first else { continue };
        if i == 0 && first == "Tool" { continue; }

        // The first column is what we need; keep it as-is
        tools.insert(first.to_string());
    }

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
