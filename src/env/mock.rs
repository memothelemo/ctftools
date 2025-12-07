use anyhow::Result;
use bon::Builder;
use dashmap::DashMap;
use std::collections::HashMap;
use std::path::PathBuf;

#[cfg(feature = "auto-install-tools")]
use std::time::Duration;

use crate::env::Environment;
use crate::registry::ToolMetadata;

#[cfg(feature = "auto-install-tools")]
use crate::install::{InstallProgress, InstallTask};
#[cfg(feature = "auto-install-tools")]
use crate::pkg::{AurHelper, PackageManager};

#[derive(Debug, Builder)]
pub struct MockEnvironment {
    #[cfg(feature = "auto-install-tools")]
    pkg_manager: Option<PackageManager>,
    #[cfg(feature = "auto-install-tools")]
    aur_helper: Option<AurHelper>,

    #[builder(default)]
    #[builder(setters(vis = "", name = installed_tools_internal))]
    installed_tools: DashMap<String, PathBuf>,

    #[builder(default = true)]
    running_in_elevation: bool,

    #[builder(default = true)]
    supports_privilege_escalation: bool,
}

impl Environment for MockEnvironment {
    fn running_in_elevation(&self) -> bool {
        self.running_in_elevation
    }

    fn supports_privilege_escalation(&self) -> bool {
        self.supports_privilege_escalation
    }

    #[cfg(feature = "auto-install-tools")]
    fn pkg_manager(&self) -> Option<(PackageManager, PathBuf)> {
        self.pkg_manager.map(|pm| (pm, PathBuf::from("")))
    }

    #[cfg(feature = "auto-install-tools")]
    fn aur_helper(&self) -> Option<(AurHelper, PathBuf)> {
        self.aur_helper.map(|pm| (pm, PathBuf::from("")))
    }

    fn find_tool_executable(&self, tool: &ToolMetadata) -> Result<Option<PathBuf>> {
        Ok(self.installed_tools.get(&tool.command).map(|v| v.clone()))
    }

    #[cfg(feature = "auto-install-tools")]
    fn run_install_task(
        &self,
        task: &InstallTask,
        progress_handler: &mut dyn FnMut(InstallProgress),
    ) -> Result<()> {
        let tool_name = task.tool_name().to_string();
        self.installed_tools
            .insert(tool_name.clone(), PathBuf::new());

        progress_handler(InstallProgress::Success {
            elapsed: Duration::ZERO,
            tool_name,
        });
        Ok(())
    }
}

impl<S: mock_environment_builder::State> MockEnvironmentBuilder<S> {
    pub fn installed_tools(
        self,
        tools: HashMap<String, PathBuf>,
    ) -> MockEnvironmentBuilder<mock_environment_builder::SetInstalledTools<S>>
    where
        S::InstalledTools: mock_environment_builder::IsUnset,
    {
        let dashmap = DashMap::new();
        for (key, value) in tools {
            dashmap.insert(key, value);
        }
        self.installed_tools_internal(dashmap)
    }
}

#[cfg(test)]
mod tests {
    use crate::env::{Environment, MockEnvironment};
    use crate::registry::{ToolMetadata, Toolkit};

    #[cfg(feature = "auto-install-tools")]
    use crate::install::{InstallPlanResult, InstallTask};
    #[cfg(feature = "auto-install-tools")]
    use crate::pkg::{AurHelper, PackageManager};

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

    #[cfg(feature = "auto-install-tools")]
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
                sudo: true,
                tool_name: "tool".to_string(),
            })
        );
    }

    #[cfg(feature = "auto-install-tools")]
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
                sudo: true,
                tool_name: "tool".to_string(),
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
                sudo: true,
                tool_name: "tool".to_string(),
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
                sudo: false,
                tool_name: "tool".to_string(),
            })
        );
    }

    #[cfg(feature = "auto-install-tools")]
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
                sudo: false,
                tool_name: "tool".to_string(),
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
                sudo: true,
                tool_name: "tool".to_string(),
            })
        );
    }
}
