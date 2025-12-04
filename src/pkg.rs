use anyhow::Result;
use cfg_if::cfg_if;
use std::path::PathBuf;

/// Represents the system's package manager.
///
/// The available options vary depending on the operating system
/// and the support from this program:
///
/// - **Windows**: `Chocolatey`, `WinGet`
/// - **macOS**: `Homebrew`
/// - **Linux**: `APT`, `DNF`, `Pacman`
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PackageManager {
    // Windows
    Chocolatey,
    WinGet,

    // macOS
    Homebrew,

    // Linux
    APT,
    DNF,
    Pacman,
}

impl PackageManager {
    /// Detects the system's preferred package manager.
    ///
    /// Returns `Ok(Some((PackageManager, PathBuf)))` if a package manager
    /// is found, where the `PathBuf` is the full path to its executable.
    ///
    /// Returns `Ok(None)` if no supported package manager is detected.
    ///
    /// Detection is performed based on the operating system and available
    /// binaries in the system PATH.
    pub fn detect() -> Result<Option<(Self, PathBuf)>> {
        cfg_if! {
            if #[cfg(target_os = "linux")] {
                Self::detect_linux()
            } else if #[cfg(target_os = "macos")] {
                Self::detect_macos()
            } else if #[cfg(target_os = "windows")] {
                Self::detect_windows()
            } else {
                Ok(None)
            }
        }
    }

    /// Returns a human-friendly name for this package manager
    /// suitable for UI display.
    #[must_use]
    pub fn as_display_name(&self) -> &'static str {
        match self {
            Self::Chocolatey => "Chocolatey",
            Self::WinGet => "WinGet",
            Self::Homebrew => "Homebrew",
            Self::APT => "APT",
            Self::DNF => "DNF",
            Self::Pacman => "Pacman",
        }
    }

    /// Returns the string key associated with this package manager in
    /// the built-in toolkit registry.
    ///
    /// This key corresponds to the identifier used internally by the program
    /// to reference tools installed or managed via the given package manager.
    #[must_use]
    pub fn as_registry_key(&self) -> &'static str {
        match self {
            Self::Chocolatey => "chocolatey",
            Self::WinGet => "winget",
            Self::Homebrew => "homebrew",
            Self::APT => "apt",
            Self::DNF => "dnf",
            Self::Pacman => "pacman",
        }
    }

    #[cfg(target_os = "macos")]
    fn detect_macos() -> Result<Option<(Self, PathBuf)>> {
        find_first_match(&[("brew", PackageManager::Homebrew)])
    }

    #[cfg(target_os = "linux")]
    fn detect_linux() -> Result<Option<(Self, PathBuf)>> {
        find_first_match(&[
            ("apt", PackageManager::APT),
            ("dnf", PackageManager::DNF),
            ("pacman", PackageManager::Pacman),
        ])
    }

    #[cfg(target_os = "windows")]
    fn detect_windows() -> Result<Option<(Self, PathBuf)>> {
        find_first_match(&[
            ("choco", PackageManager::Chocolatey),
            ("winget", PackageManager::WinGet),
        ])
    }
}

impl PackageManager {
    /// Returns whether this package manager requires elevated privileges
    /// (e.g., root or administrator) to install or update packages.
    #[must_use]
    pub const fn needs_privilege(&self) -> bool {
        match self {
            Self::Chocolatey | Self::WinGet | Self::Homebrew => false,
            Self::APT | Self::DNF | Self::Pacman => true,
        }
    }
}

/// Represents an AUR helper, which are user-space wrappers for Pacman
/// commonly used on Arch Linux distributions.
///
/// You may find other AUR helpers that are not supported
/// in this program at: https://wiki.archlinux.org/title/AUR_helpers#Pacman_wrappers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AurHelper {
    Paru,
    Yay,
}

impl AurHelper {
    /// Detects the system's preferred AUR helper.
    ///
    /// Returns `Ok(Some((PackageManager, PathBuf)))` if a AUR helper
    /// is found, where the `PathBuf` is the full path to its executable.
    ///
    /// Returns `Ok(None)` if no supported AUR helper is detected.
    ///
    /// Detection is performed based on the available binaries in the system PATH.
    pub fn detect() -> Result<Option<(Self, PathBuf)>> {
        find_first_match(&[("paru", Self::Paru), ("yay", Self::Yay)])
    }

    /// Returns whether this AUR helper requires elevated privileges.
    ///
    /// AUR helpers are usually operate in user-space by default.
    #[must_use]
    pub const fn needs_privilege(&self) -> bool {
        false
    }
}

/// Searches the system for the first matching the binary.
fn find_first_match<T: Copy>(candidates: &[(&str, T)]) -> Result<Option<(T, PathBuf)>> {
    use crate::util::which_opt;

    for (cmd, pm) in candidates {
        if let Some(path) = which_opt(cmd)? {
            return Ok(Some((*pm, path)));
        }
    }

    Ok(None)
}
