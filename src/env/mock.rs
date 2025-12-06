use anyhow::Result;
use bon::Builder;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::env::Environment;
use crate::pkg::{AurHelper, PackageManager};
use crate::registry::ToolMetadata;

#[derive(Debug, Builder)]
pub struct MockEnvironment {
    pkg_manager: Option<PackageManager>,
    aur_helper: Option<AurHelper>,

    #[builder(default)]
    installed_tools: HashMap<String, PathBuf>,
}

impl Environment for MockEnvironment {
    fn pkg_manager(&self) -> Option<(PackageManager, PathBuf)> {
        self.pkg_manager.map(|pm| (pm, PathBuf::from("")))
    }

    fn aur_helper(&self) -> Option<(AurHelper, PathBuf)> {
        self.aur_helper.map(|pm| (pm, PathBuf::from("")))
    }

    fn find_tool_executable(&self, tool: &ToolMetadata) -> Result<Option<PathBuf>> {
        Ok(self.installed_tools.get(&tool.command).cloned())
    }
}

#[cfg(test)]
mod tests {
    use crate::env::{Environment, MockEnvironment};
    use crate::install_task::{InstallPlanResult, InstallTask};
    use crate::pkg::{AurHelper, PackageManager};
    use crate::registry::{ToolMetadata, Toolkit};

    use maplit::hashmap;
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;
    use std::sync::LazyLock;

    static SAMPLE_TOOLKIT: LazyLock<Toolkit> = LazyLock::new(|| {
        let tool1 = ToolMetadata::builder()
            .name("foo".into())
            .command("foo".into())
            .build();

        Toolkit::new(vec![tool1])
    });

    #[test]
    fn test_check_toolkit_installation() {
        // Test #1: Regular usage
        let env = MockEnvironment::builder()
            .installed_tools(hashmap! {
                "foo".to_string() => PathBuf::from("bar"),
            })
            .build();

        let missing_tools = env.check_toolkit_installation(&SAMPLE_TOOLKIT).unwrap();
        let (_, installed) = missing_tools
            .iter()
            .find(|(tool, ..)| tool.name == "foo")
            .unwrap();

        assert!(installed);

        // Test #2: The tool is not installed
        let env = MockEnvironment::builder().build();

        let missing_tools = env.check_toolkit_installation(&SAMPLE_TOOLKIT).unwrap();
        let (_, installed) = missing_tools
            .iter()
            .find(|(tool, ..)| tool.name == "foo")
            .unwrap();

        assert!(!installed);
    }

    #[test]
    fn test_find_tool_executable() {
        let path = PathBuf::from("/usr/bin/ping");
        let env = MockEnvironment::builder()
            .installed_tools(hashmap! {
                "ping".to_string() => path.clone(),
            })
            .build();

        let tool = ToolMetadata::builder()
            .name("ping".into())
            .command("ping".into())
            .build();

        assert_eq!(env.find_tool_executable(&tool).unwrap(), Some(path));

        let non_existing_tool = ToolMetadata::builder()
            .name("pong".into())
            .command("pong".into())
            .build();

        assert_eq!(env.find_tool_executable(&non_existing_tool).unwrap(), None);
    }

    #[test]
    fn test_plan_install_tool_with_provided_default_package() {
        let tool = ToolMetadata::builder()
            .name("tool".into())
            .command("tool".into())
            .packages(hashmap! {
                "default".to_string() => "tool".to_string()
            })
            .build();

        let env = MockEnvironment::builder()
            .pkg_manager(PackageManager::Pacman)
            .build();

        assert_eq!(
            env.plan_install_tool(&tool),
            InstallPlanResult::Task(InstallTask::PackageManager {
                exec: PathBuf::from(""),
                arguments: ["-S", "--noconfirm", "tool"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>(),
                sudo: true
            })
        );
    }

    #[test]
    fn test_plan_install_tool_with_specific_package_names() {
        let tool = ToolMetadata::builder()
            .name("tool".into())
            .command("tool".into())
            .packages(hashmap! {
                "default".to_string() => "tool".to_string(),
                "apt".to_string() => "tool-debian".to_string(),
                "pacman".to_string() => "tool-pacman".to_string(),
            })
            .build();

        // Pacman test //
        let env = MockEnvironment::builder()
            .pkg_manager(PackageManager::Pacman)
            .build();

        assert_eq!(
            env.plan_install_tool(&tool),
            InstallPlanResult::Task(InstallTask::PackageManager {
                exec: PathBuf::from(""),
                arguments: ["-S", "--noconfirm", "tool-pacman"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>(),
                sudo: true
            })
        );

        // APT test //
        let env = MockEnvironment::builder()
            .pkg_manager(PackageManager::APT)
            .build();

        assert_eq!(
            env.plan_install_tool(&tool),
            InstallPlanResult::Task(InstallTask::PackageManager {
                exec: PathBuf::from(""),
                arguments: ["install", "-y", "tool-debian"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>(),
                sudo: true
            })
        );

        // Default test //
        let env = MockEnvironment::builder()
            .pkg_manager(PackageManager::Chocolatey)
            .build();

        assert_eq!(
            env.plan_install_tool(&tool),
            InstallPlanResult::Task(InstallTask::PackageManager {
                exec: PathBuf::from(""),
                arguments: ["install", "tool", "-y"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>(),
                sudo: false
            })
        );
    }

    #[test]
    fn test_plan_install_tool_with_aur_support() {
        // Test case: if we don't have an equivalent package in pacman
        let tool = ToolMetadata::builder()
            .name("tool".into())
            .command("tool".into())
            .packages(hashmap! {
                "aur".to_string() => "tool-bin".to_string(),
            })
            .build();

        let env = MockEnvironment::builder()
            .pkg_manager(PackageManager::Pacman)
            .aur_helper(AurHelper::Paru)
            .build();

        assert_eq!(
            env.plan_install_tool(&tool),
            InstallPlanResult::Task(InstallTask::PackageManager {
                exec: PathBuf::from(""),
                arguments: ["-S", "tool-bin"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>(),
                sudo: false
            })
        );

        // Test case: if we do have an equivalent package in pacman
        let tool = ToolMetadata::builder()
            .name("tool".into())
            .command("tool".into())
            .packages(hashmap! {
                "pacman".to_string() => "tool-oss".to_string(),
                "aur".to_string() => "tool-bin".to_string(),
            })
            .build();

        let env = MockEnvironment::builder()
            .pkg_manager(PackageManager::Pacman)
            .aur_helper(AurHelper::Paru)
            .build();

        assert_eq!(
            env.plan_install_tool(&tool),
            InstallPlanResult::Task(InstallTask::PackageManager {
                exec: PathBuf::from(""),
                arguments: ["-S", "--noconfirm", "tool-oss"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>(),
                sudo: true
            })
        );
    }
}
