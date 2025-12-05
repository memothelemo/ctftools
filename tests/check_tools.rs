use assert_cmd::{Command, cargo_bin};
use ctftools::registry::{ToolMetadata, Toolkit};

#[test]
fn test_empty_toolkit() {
    let cmd = Command::new(cargo_bin!("ctftools"))
        .args(["--custom-toolkit", "{}"])
        .arg("check")
        .unwrap();

    let output = String::from_utf8_lossy(&cmd.stdout);
    insta::assert_snapshot!(output);
}

#[test]
fn test_missing_toolkit() {
    // ping exists in both Windows and Unix systems
    let tool = ToolMetadata::builder()
        .name("unknown-tool".into())
        .command("unknown-tool".into())
        .build();

    let toolkit = Toolkit::new(vec![tool]).serialize_into_json();
    let cmd = Command::new(cargo_bin!("ctftools"))
        .args(["--custom-toolkit", &*toolkit])
        .arg("check")
        .unwrap();

    let output = String::from_utf8_lossy(&cmd.stdout);
    insta::assert_snapshot!(output);
}

#[test]
fn test_existing_toolkit() {
    // ping exists in both Windows and Unix systems
    let tool = ToolMetadata::builder()
        .name("ping".into())
        .command("ping".into())
        .build();

    let toolkit = Toolkit::new(vec![tool]).serialize_into_json();
    let cmd = Command::new(cargo_bin!("ctftools"))
        .args(["--custom-toolkit", &*toolkit])
        .arg("check")
        .unwrap();

    let output = String::from_utf8_lossy(&cmd.stdout);
    insta::assert_snapshot!(output);
}
