use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "auto-install-tools")] {
        use assert_cmd::{Command, cargo_bin};
        use cfg_if::cfg_if;
        use ctftools::registry::{ToolMetadata, Toolkit};
        use std::process::Output;

        fn run_install_tools(installed_tools: &[&str], toolkit: &Toolkit) -> Output {
            let toolkit = toolkit.serialize_into_json();
            let installed_tools = installed_tools.join(",");

            Command::new(cargo_bin!("ctftools"))
                .args(["--custom-toolkit", &*toolkit])
                .args(["--mock-installed-tools", &*installed_tools])
                .arg("install")
                .unwrap()
        }

        #[test]
        fn test_empty_toolkit() {
            let cmd = run_install_tools(&[], &Toolkit::new(Vec::new()));
            let stdout = String::from_utf8_lossy(&cmd.stdout);
            insta::assert_snapshot!(stdout);
        }

        #[test]
        fn test_could_not_install() {
            let tool = ToolMetadata::builder()
                .name("foo".into())
                .command("foo".into())
                .build();

            let toolkit = Toolkit::new(vec![tool]);
            let cmd = run_install_tools(&[], &toolkit);

            let output = String::from_utf8_lossy(&cmd.stderr);
            let output = anstream::adapter::strip_str(&output);
            insta::assert_snapshot!(output);
        }
    }
}
