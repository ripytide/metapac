use assert_cmd::{assert::OutputAssertExt, cargo::CommandCargoExt};
use markdown::{ParseOptions, mdast::Node};
use std::process::Command;

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

    let mut cmd = Command::cargo_bin("metapac").unwrap();
    cmd.args(["--hostname", "pc", "--config-dir", ".", "unmanaged"]);
    cmd.assert().success();

    std::fs::remove_dir_all("groups").unwrap();
    std::fs::remove_file("config.toml").unwrap();
}
