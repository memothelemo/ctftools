use assert_cmd::{Command, cargo_bin};
use ctftools::registry::{ToolMetadata, Toolkit};
use std::process::Output;

fn run_check_tools(installed_tools: &[&str], toolkit: &Toolkit) -> Output {
    let toolkit = toolkit.serialize_into_yml();
    let installed_tools = installed_tools.join(",");

    Command::new(cargo_bin!("ctftools"))
        .args(["--custom-toolkit", &*toolkit])
        .args(["--mock-installed-tools", &*installed_tools])
        .arg("check")
        .unwrap()
}

#[test]
fn test_empty_toolkit() {
    let cmd = run_check_tools(&[], &Toolkit::new(Vec::new()));
    let stdout = String::from_utf8_lossy(&cmd.stdout);
    insta::assert_snapshot!(stdout);
}

#[test]
fn test_installed_all_tools_from_toolkit() {
    let tool = ToolMetadata::builder()
        .name("foo".into())
        .command("foo".into())
        .build();

    let toolkit = Toolkit::new(vec![tool]);
    let cmd = run_check_tools(&["foo"], &toolkit);

    let output = String::from_utf8_lossy(&cmd.stdout);
    insta::assert_snapshot!(output);
}

#[test]
fn test_missing_tools_from_toolkit() {
    let toolkit = Toolkit::new(vec![
        ToolMetadata::builder()
            .name("intangible".into())
            .command("intangible".into())
            .build(),
        ToolMetadata::builder()
            .name("tangible".into())
            .command("tangible".into())
            .build(),
    ]);

    let cmd = run_check_tools(&["tangible"], &toolkit);
    let output = String::from_utf8_lossy(&cmd.stdout);
    insta::assert_snapshot!(output);
}
