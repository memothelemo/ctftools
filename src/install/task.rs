use serde::Serialize;
use std::path::PathBuf;
use thiserror::Error;

use crate::pkg::{AurHelper, PackageManager};
use crate::registry::{ToolDownloadInstructions, ToolMetadata};

/// Represents an action to install a tool.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub enum InstallTask {
    /// Install the tool using a system package manager executable.
    PackageManager {
        /// Path to the package manager executable (e.g. `/usr/bin/apt`).
        exec: PathBuf,

        /// Arguments to pass to the package manager.
        arguments: Vec<String>,

        /// Whether the package manager invocation requires elevated privileges.
        sudo: bool,

        /// The original tool name to be installed.
        tool_name: String,
    },

    /// Install by downloading an installer from a URL.
    Download {
        /// Instructions on how to install a tool from a download.
        instructions: ToolDownloadInstructions,

        /// The original tool name to be installed.
        tool_name: String,
    },

    /// Install the tool by installing a package from the Arch
    /// User Repository (AUR) with `makepkg -si`.
    ///
    /// This variant can be produced by [`InstallTask::from_package_manager`],
    /// if so, be sure call to call [`InstallTask::from_aur`] to convert it
    /// into [`InstallTask::PackageManager`] that will run the appropriate install
    /// command for the chosen AUR helper.
    ///
    /// This task is only applicable on Arch Linux.
    AUR {
        /// Name of the package in the AUR.
        package_name: String,

        /// The original tool name to be installed.
        tool_name: String,
    },
}

impl InstallTask {
    /// Gets the associated tool name from a task in any variant.
    #[must_use]
    pub fn tool_name(&self) -> &str {
        match self {
            Self::AUR { tool_name, .. } => tool_name,
            Self::Download { tool_name, .. } => tool_name,
            Self::PackageManager { tool_name, .. } => tool_name,
        }
    }
}

/// Errors that can occur while creating an [`InstallTask`] from a tool.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum InstallTaskError {
    /// No automatic install method is available for the tool.
    #[error("{tool_name:?} cannot be installed automatically")]
    CannotInstallTool { tool_name: String },

    /// The tool is not available in the Arch User Repository (AUR).
    #[error("Cannot find AUR equivalent package for {tool_name:?}")]
    AurPackageNotFound {
        pkg_manager: PackageManager,
        tool_name: String,
    },

    /// The tool is not available in the requested package manager's registry.
    #[error("Cannot find {} equivalent package for {tool_name:?}", .pkg_manager.as_display_name())]
    PackageNotFound {
        pkg_manager: PackageManager,
        tool_name: String,
    },
}

impl InstallTask {
    /// Creates an appropriate [`InstallTask`] object from a
    /// specific AUR helper to install a provided tool.
    #[must_use]
    pub fn from_aur(
        aur_helper: AurHelper,
        path_to_aur_helper: PathBuf,
        package_name: String,
        tool_name: String,
    ) -> Self {
        let arguments = match aur_helper {
            AurHelper::Paru | AurHelper::Yay => ["-S", &*package_name],
        }
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();

        Self::PackageManager {
            exec: path_to_aur_helper,
            arguments,
            sudo: aur_helper.needs_privilege(),
            tool_name,
        }
    }

    /// Create an [`InstallTask`] from the tool's download metadata.
    ///
    /// This prefers platform-specific download entries. If no matching
    /// download URL exists for the current target OS, it returns
    /// `Err(InstallTaskError::CannotInstallTool)`.
    pub fn from_downloads(tool: &ToolMetadata) -> Result<Self, InstallTaskError> {
        let instructions = if cfg!(target_os = "windows") {
            tool.downloads.windows.clone()
        } else if cfg!(target_os = "macos") {
            tool.downloads.macos.clone()
        } else if cfg!(target_os = "linux") {
            tool.downloads.linux.clone()
        } else {
            None
        };

        instructions
            .map(|inner| Self::Download {
                instructions: inner,
                tool_name: tool.name.clone(),
            })
            .ok_or_else(|| InstallTaskError::CannotInstallTool {
                tool_name: tool.name.clone(),
            })
    }

    /// Creates an appropriate [`InstallTask`] object from
    /// a specific package manager to install a provided tool.
    ///
    /// For Pacman, this function will prefer pacman-specific packages, fall back
    /// to AUR packages if present, or use its pacman-supported package.
    pub fn from_package_manager(
        pkg_manager: PackageManager,
        path_to_pkg_manager: PathBuf,
        tool: &ToolMetadata,
    ) -> Result<Self, InstallTaskError> {
        // Handle Pacman separately because it may need the AUR helper.
        if pkg_manager == PackageManager::Pacman {
            // Look for pacman, aur, or default packages
            let mut pkg_name = tool.packages.get("pacman");
            let mut use_aur = false;

            if pkg_name.is_none() {
                pkg_name = tool.packages.get("aur");
                use_aur = pkg_name.is_some();
            }

            // Or maybe in the defaults?
            if pkg_name.is_none() {
                pkg_name = tool.packages.get("default");
                use_aur = false;
            }

            let Some(arch_package) = pkg_name else {
                return Err(InstallTaskError::PackageNotFound {
                    pkg_manager,
                    tool_name: tool.name.clone(),
                });
            };

            if use_aur {
                return Ok(InstallTask::AUR {
                    package_name: arch_package.to_string(),
                    tool_name: tool.name.clone(),
                });
            }

            let arguments = ["-S", "--noconfirm", arch_package]
                .into_iter()
                .map(String::from)
                .collect();

            return Ok(InstallTask::PackageManager {
                exec: path_to_pkg_manager,
                arguments,
                sudo: pkg_manager.needs_privilege(),
                tool_name: tool.name.clone(),
            });
        }

        let package_name = tool
            .packages
            .get(pkg_manager.as_registry_key())
            .or_else(|| tool.packages.get("default"))
            .ok_or_else(|| InstallTaskError::PackageNotFound {
                pkg_manager,
                tool_name: tool.name.clone(),
            })?;

        let args = match pkg_manager {
            PackageManager::APT => ["install", "-y", package_name],
            PackageManager::DNF => ["install", "-y", package_name],
            PackageManager::Homebrew => ["install", package_name, ""],
            PackageManager::Chocolatey => ["install", package_name, "-y"],
            PackageManager::WinGet => ["install", package_name, "--accept-package-agreements"],
            PackageManager::Pacman => unreachable!(),
        }
        .into_iter()
        .map(String::from)
        .collect();

        Ok(InstallTask::PackageManager {
            exec: path_to_pkg_manager,
            arguments: args,
            sudo: pkg_manager.needs_privilege(),
            tool_name: tool.name.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use maplit::hashmap;
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;

    use crate::install::{InstallTask, InstallTaskError};
    use crate::pkg::PackageManager;
    use crate::registry::{
        DownloadFileFormat, ToolDownloadInstructions, ToolMetadata, ToolPlatformDownloads,
    };

    #[test]
    fn test_from_download_with_no_download_links() {
        let tool = ToolMetadata::builder()
            .name("foo".to_string())
            .command("foo".to_string())
            .build();

        let result = InstallTask::from_downloads(&tool);
        assert_eq!(
            result,
            Err(InstallTaskError::CannotInstallTool {
                tool_name: "foo".to_string()
            })
        );
    }

    #[cfg_attr(
        any(target_os = "windows", target_os = "macos", target_os = "linux"),
        test
    )]
    fn test_from_download_with_download_links() {
        let expected_link = if cfg!(target_os = "windows") {
            "https://foo.local/downloads/windows.exe"
        } else if cfg!(target_os = "macos") {
            "https://foo.local/downloads/macos.exe"
        } else if cfg!(target_os = "linux") {
            "https://foo.local/downloads/linux.exe"
        } else {
            unreachable!()
        };

        let tool = ToolMetadata::builder()
            .name("foo".to_string())
            .command("foo".to_string())
            .downloads(
                ToolPlatformDownloads::builder()
                    .windows(
                        ToolDownloadInstructions::builder()
                            .url("https://foo.local/downloads/windows.exe".to_string())
                            .format(DownloadFileFormat::Executable)
                            .build(),
                    )
                    .macos(
                        ToolDownloadInstructions::builder()
                            .url("https://foo.local/downloads/macos.dmg".to_string())
                            .format(DownloadFileFormat::Executable)
                            .build(),
                    )
                    .linux(
                        ToolDownloadInstructions::builder()
                            .url("https://foo.local/downloads/linux".to_string())
                            .format(DownloadFileFormat::Executable)
                            .build(),
                    )
                    .build(),
            )
            .build();

        let result = InstallTask::from_downloads(&tool);
        assert_eq!(
            result,
            Ok(InstallTask::Download {
                instructions: ToolDownloadInstructions::builder()
                    .url(expected_link.to_string())
                    .format(DownloadFileFormat::Executable)
                    .build(),
                tool_name: "foo".to_string(),
            })
        );
    }

    #[test]
    fn test_other_package_managers_with_default_field() {
        struct TestCase {
            package_manager: PackageManager,
            expected_package: &'static str,
        }

        let tool = ToolMetadata::builder()
            .name("foo".to_string())
            .command("foo".to_string())
            .packages(hashmap! {
                "default".to_string() => "foo-default".to_string(),
                "apt".to_string() => "foo-debian".to_string(),
                "homebrew".to_string() => "foo-macos".to_string(),
                "chocolatey".to_string() => "foo-win".to_string(),
            })
            .build();

        let cases = &[
            TestCase {
                package_manager: PackageManager::APT,
                expected_package: "foo-debian",
            },
            TestCase {
                package_manager: PackageManager::DNF,
                expected_package: "foo-default",
            },
            TestCase {
                package_manager: PackageManager::Homebrew,
                expected_package: "foo-macos",
            },
            TestCase {
                package_manager: PackageManager::Chocolatey,
                expected_package: "foo-win",
            },
            TestCase {
                package_manager: PackageManager::WinGet,
                expected_package: "foo-default",
            },
        ];

        for case in cases {
            let result = InstallTask::from_package_manager(
                case.package_manager,
                PathBuf::from("this argument is not strictly evaluated"),
                &tool,
            );

            eprintln!(
                "Testing with {:?} package manager (expected package: {:?})",
                case.package_manager, case.expected_package
            );
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_other_package_managers_with_no_default_field() {
        struct TestCase {
            package_manager: PackageManager,
            expected_package: &'static str,
        }

        let tool = ToolMetadata::builder()
            .name("foo".to_string())
            .command("foo".to_string())
            .packages(hashmap! {
                "apt".to_string() => "foo-debian".to_string(),
                "dnf".to_string() => "foo-dnf".to_string(),
                "homebrew".to_string() => "foo-macos".to_string(),
                "chocolatey".to_string() => "foo-win".to_string(),
                "winget".to_string() => "foo-win".to_string(),
            })
            .build();

        let cases = &[
            TestCase {
                package_manager: PackageManager::APT,
                expected_package: "foo-debian",
            },
            TestCase {
                package_manager: PackageManager::DNF,
                expected_package: "foo-dnf",
            },
            TestCase {
                package_manager: PackageManager::Homebrew,
                expected_package: "foo-macos",
            },
            TestCase {
                package_manager: PackageManager::Chocolatey,
                expected_package: "foo-win",
            },
            TestCase {
                package_manager: PackageManager::WinGet,
                expected_package: "foo-win",
            },
        ];

        for case in cases {
            let result = InstallTask::from_package_manager(
                case.package_manager,
                PathBuf::from("this argument is not strictly evaluated"),
                &tool,
            );

            eprintln!(
                "Testing with {:?} package manager (expected package: {:?})",
                case.package_manager, case.expected_package
            );
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_pacman() {
        let tool = ToolMetadata::builder()
            .name("foo".to_string())
            .command("foo".to_string())
            .packages(hashmap! {
                "default".to_string() => "foo".to_string()
            })
            .build();

        let result = InstallTask::from_package_manager(
            PackageManager::Pacman,
            PathBuf::from("/usr/bin/pacman"),
            &tool,
        );

        // It should throw an error because we haven't declared
        // the packages map to include `default` one.
        assert_eq!(
            result,
            Ok(InstallTask::PackageManager {
                exec: PathBuf::from("/usr/bin/pacman"),
                arguments: ["-S", "--noconfirm", "foo"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                sudo: true,
                tool_name: "foo".to_string(),
            })
        );
    }

    #[test]
    fn test_pacman_with_specific_pacman_package() {
        let tool = ToolMetadata::builder()
            .name("foo".to_string())
            .command("foo".to_string())
            .packages(hashmap! {
                "default".to_string() => "foo".to_string(),
                "pacman".to_string() => "foo-pacman".to_string()
            })
            .build();

        let result = InstallTask::from_package_manager(
            PackageManager::Pacman,
            PathBuf::from("/usr/bin/pacman"),
            &tool,
        );

        // It should throw an error because we haven't declared
        // the packages map to include `default` one.
        assert_eq!(
            result,
            Ok(InstallTask::PackageManager {
                exec: PathBuf::from("/usr/bin/pacman"),
                arguments: ["-S", "--noconfirm", "foo-pacman"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                sudo: true,
                tool_name: "foo".to_string(),
            })
        );
    }

    #[test]
    fn test_pacman_needs_aur_installation() {
        let tool = ToolMetadata::builder()
            .name("foo".to_string())
            .command("foo".to_string())
            .packages(hashmap! {
                "aur".to_string() => "foo-bin".to_string()
            })
            .build();

        let result = InstallTask::from_package_manager(
            PackageManager::Pacman,
            PathBuf::from("/usr/bin/pacman"),
            &tool,
        );

        // It should throw an error because we haven't declared
        // the packages map to include `default` one.
        assert_eq!(
            result,
            Ok(InstallTask::AUR {
                package_name: "foo-bin".to_string(),
                tool_name: "foo".to_string(),
            })
        );
    }

    #[test]
    fn test_pacman_with_no_default_pkg() {
        let tool = ToolMetadata::builder()
            .name("foo".to_string())
            .command("foo".to_string())
            .build();

        let result = InstallTask::from_package_manager(
            PackageManager::Pacman,
            PathBuf::from("/usr/bin/pacman"),
            &tool,
        );

        // It should throw an error because we haven't declared
        // the packages map to include `default` one.
        assert_eq!(
            result,
            Err(InstallTaskError::PackageNotFound {
                pkg_manager: PackageManager::Pacman,
                tool_name: "foo".to_string()
            })
        );
    }
}
