use assert_cmd::cargo::cargo_bin_cmd;
use markdown::{ParseOptions, mdast::Node};

fn assert_help_contains(args: &[&str], needle: &str) {
    let output = cargo_bin_cmd!().args(args).output().unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(needle),
        "{stdout:?} did not contain {needle:?}"
    );
}

#[test]
fn refresh_help() {
    assert_help_contains(&["refresh", "--help"], "refresh package metadata");
}

#[test]
fn sync_update_and_update_all_have_refresh_flag() {
    assert_help_contains(&["sync", "--help"], "--refresh");
    assert_help_contains(&["update", "--help"], "--refresh");
    assert_help_contains(&["update-all", "--help"], "--refresh");
}

#[test]
fn unmanaged() {
    let readme =
        markdown::to_mdast(include_str!("../README.md"), &ParseOptions::default()).unwrap();

    let mut nodes = Vec::new();
    nodes.append(&mut readme.children().unwrap().clone());

    let mut leaves = Vec::new();

    while let Some(node) = nodes.pop() {
        if let Some(children) = node.children() {
            nodes.append(&mut children.clone());
        } else {
            leaves.push(node.clone());
        }
    }

    let toml_blocks = leaves
        .iter()
        .filter_map(|x| {
            if let Node::Code(code) = x {
                Some(code)
            } else {
                None
            }
        })
        .filter(|code| code.lang == Some("toml".to_string()))
        .collect::<Vec<_>>();

    let config = &toml_blocks[1].value;
    let group = &toml_blocks[0].value;

    std::fs::write("config.toml", config).unwrap();
    std::fs::create_dir("groups").unwrap();
    std::fs::write("groups/example_group.toml", group).unwrap();

    let mut cmd = cargo_bin_cmd!();
    cmd.args(["--hostname", "pc", "--config-dir", ".", "unmanaged"]);
    cmd.assert().success();

    std::fs::remove_dir_all("groups").unwrap();
    std::fs::remove_file("config.toml").unwrap();
}
