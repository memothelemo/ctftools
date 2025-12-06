use anyhow::Result;
use std::path::PathBuf;

use crate::env::Environment;
use crate::pkg::{AurHelper, PackageManager};
use crate::registry::ToolMetadata;
use crate::util::which_opt;

#[derive(Debug)]
pub struct LiveEnvironment {
    pkg_manager: Option<WithPath<PackageManager>>,
    aur_helper: Option<WithPath<AurHelper>>,
}

impl LiveEnvironment {
    /// Creates a new [`LiveEnvironment`] where it detects available
    /// system package managers and AUR helper (if the user installed
    /// Arch Linux or has an AUR helper binary present)
    pub fn new() -> Result<Self> {
        Ok(Self {
            pkg_manager: PackageManager::detect()?.map(Into::into),
            aur_helper: AurHelper::detect()?.map(Into::into),
        })
    }

    /// Creates a new [`LiveEnvironment`] with a package manager present.
    ///
    /// [AUR helper] will not be present at all times once created.
    #[must_use]
    pub fn with_pkg_manager(pm: PackageManager, path: PathBuf) -> Self {
        Self {
            pkg_manager: Some(WithPath { inner: pm, path }),
            aur_helper: None,
        }
    }

    /// Creates a new [`LiveEnvironment`] with no package manager present.
    #[must_use]
    pub fn without_pkg_manager() -> Self {
        Self {
            pkg_manager: None,
            aur_helper: None,
        }
    }
}

impl Environment for LiveEnvironment {
    fn is_live(&self) -> bool {
        true
    }

    fn pkg_manager(&self) -> Option<(PackageManager, PathBuf)> {
        self.pkg_manager.as_ref().cloned().map(WithPath::into_inner)
    }

    fn aur_helper(&self) -> Option<(AurHelper, PathBuf)> {
        self.aur_helper.as_ref().cloned().map(WithPath::into_inner)
    }

    /// Attempts to locate the executable for a specific tool
    /// described by [`ToolMetadata`]
    ///
    /// The lookup strategy for [`LiveEnvironment`] is:
    /// 1. Try to find the command on the system `PATH`.
    /// 2. On Windows, also check any additional executable paths
    ///    associated with the tool's metadata.
    fn find_tool_executable(&self, tool: &ToolMetadata) -> Result<Option<PathBuf>> {
        // There are ways we can find the tool executable either:
        // 1. By using the `which` operation (from PATH environment variable)
        if let Some(path) = which_opt(&tool.command)? {
            return Ok(Some(path));
        }

        // 2. Checking tool's associated executable (if the operating system is running on Windows)
        #[cfg(target_os = "windows")]
        for path in tool.windows.exec_paths.iter() {
            use anyhow::Context;

            let exists = std::fs::exists(path)
                .with_context(|| format!("failed to find {} executable", path.display()))?;

            if exists {
                return Ok(Some(path.to_path_buf()));
            }
        }

        Ok(None)
    }
}

#[derive(Clone)]
struct WithPath<T> {
    inner: T,
    path: PathBuf,
}

impl<T> WithPath<T> {
    #[must_use]
    pub fn into_inner(self) -> (T, PathBuf) {
        (self.inner, self.path)
    }
}

impl<T> From<(T, PathBuf)> for WithPath<T> {
    fn from((inner, path): (T, PathBuf)) -> Self {
        Self { inner, path }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for WithPath<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entry(&"inner", &self.inner)
            .entry(&"path", &self.path)
            .finish()
    }
}

impl<T> std::ops::Deref for WithPath<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::env::live::WithPath;
    use crate::pkg::PackageManager;

    #[cfg(windows)]
    #[test]
    fn test_find_tool_executable_in_windows() {
        use crate::env::{Environment, LiveEnvironment};
        use crate::registry::{ToolMetadata, ToolWindowsMetadata};

        let diskpart_path = PathBuf::from("C:\\Windows\\System32\\diskpart.exe");
        let env = LiveEnvironment::without_pkg_manager();

        let diskpart = ToolMetadata::builder()
            .name("diskpart".into())
            .command("".into())
            .windows(
                ToolWindowsMetadata::builder()
                    .exec_paths(vec![diskpart_path.clone()])
                    .build(),
            )
            .build();

        assert_eq!(
            env.find_tool_executable(&diskpart).unwrap(),
            Some(diskpart_path)
        );
    }

    #[test]
    fn with_path_test_debug_fmt() {
        insta::assert_debug_snapshot!(WithPath {
            inner: PackageManager::Pacman,
            path: PathBuf::from(""),
        });
    }
}
