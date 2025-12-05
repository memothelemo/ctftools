use anyhow::{Context, Result};
use bon::Builder;
use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::{Value as Json, json};
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::LazyLock;

/// A collection of tool definitions that make up the user's toolkit.
///
/// The [`Toolkit`] struct represents a set of external CTF tools that the program
/// knows about. These definitions are loaded from a compile-time bundled JSON
/// file and contain metadata describing each tool—its command, description,
/// supported package managers, and platform-specific details.
///
/// The built-in toolkit acts as the program’s predefined tool registry. Tools
/// themselves are *not* built into the binary; instead, the toolkit describes
/// external utilities that may be installed or invoked by the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toolkit {
    tools: Vec<ToolMetadata>,
}

impl Toolkit {
    /// This function creates a new toolkit based on a pre-defined
    /// list of tools used for testing the entire program.
    #[must_use]
    pub fn new(tools: Vec<ToolMetadata>) -> Self {
        Self { tools }
    }

    /// Deserializes the JSON from a given string into a toolkit.
    pub fn from_json(json: &str) -> Result<Self> {
        let map: BTreeMap<String, Json> = serde_json::from_str(json)
            .expect("failed to deserialize built-in toolkit from JSON payload");

        let mut tools = Vec::new();
        for (command, metadata) in map {
            // Ignore the _comment key because it contains invalid schema on it.
            if command == "_comment" {
                continue;
            }

            let mut tool: ToolMetadata = match serde_json::from_value(metadata) {
                Ok(okay) => okay,
                Err(error) => panic!("failed to deserialize tool {command:?}: {error:#?}"),
            };

            // Use the associated key for a name if the name field feels empty.
            if tool.name.is_empty() || tool.name.chars().all(|v| v.is_whitespace()) {
                tool.name = command.clone();
            }

            // If `default` key is not defined in packages field then insert it
            // with the associated key as a value.
            if !tool.packages.contains_key("default") {
                tool.packages.insert("default".to_string(), command.clone());
            }

            tool.command = command;
            tool.description = tool.description.trim().to_string();
            tools.push(tool);
        }

        Ok(Self { tools })
    }

    /// Returns a static reference to the predefined, compile-time bundled toolkit.
    ///
    /// This function lazily loads and deserializes the JSON file located at
    /// `assets/default/toolkit.json` in the program repository, then caches
    /// the result for all future calls. The loaded data defines the default
    /// CTF tool registry shipped with the program.
    ///
    /// This function may panic if there's something wrong with the
    /// deserialization process from `assets/default/toolkit.json` file.
    #[allow(clippy::should_implement_trait)]
    #[must_use]
    pub fn default() -> &'static Self {
        static INNER_VALUE: LazyLock<Toolkit> = LazyLock::new(|| {
            let toolkit = Toolkit::from_json(include_str!("../../assets/default/toolkit.json"))
                .context("failed to load built-in default toolkit.json")
                .unwrap();

            for tool in toolkit.tools() {
                debug!("found built-in tool: {tool:?}");
            }

            debug!(
                "successfully loaded built-in toolkit; loaded {} tool(s)",
                toolkit.tools().len()
            );
            toolkit
        });

        &INNER_VALUE
    }

    /// Returns the list of tools defined in this toolkit.
    ///
    /// Provides read-only access to all tool metadata entries. Each entry
    /// corresponds directly to one tool defined in the toolkit source.
    #[must_use]
    pub fn tools(&self) -> &[ToolMetadata] {
        &self.tools
    }

    /// Attempts to serialize into a format that follows with
    /// `assets/default/toolkit.json` in the program repository.
    #[must_use]
    pub fn serialize_into_json(&self) -> String {
        let mut map = HashMap::new();
        for tool in self.tools.iter() {
            let mut value = json!({
                "description": tool.description,
                "packages": tool.packages,
                "windows": tool.windows,
                "downloads": tool.downloads,
            });

            if !tool.name.is_empty() {
                let object = value.as_object_mut().unwrap();
                object.insert("name".to_string(), tool.name.clone().into());
            }

            map.insert(tool.command.clone(), value);
        }
        serde_json::to_string(&map).unwrap()
    }
}

/// Metadata describing a tool provided by a toolkit.
///
/// This struct carries the information needed to identify, display and
/// install or invoke a tool exposed by a toolkit.
#[derive(Debug, Deserialize, Builder, Clone, PartialEq, Eq)]
pub struct ToolMetadata {
    /// The full name of the provided tool from the toolkit
    #[serde(default)]
    pub name: String,

    /// The command or invocation used to run the tool
    #[serde(default)]
    pub command: String,

    /// A short, human-readable description summarizing the tool
    #[builder(default)]
    pub description: String,

    /// A mapping from package manager identifier as a key to its
    /// equivalent package manager that provides the tool for that
    /// package manager.
    #[builder(default)]
    #[serde(default)]
    pub packages: HashMap<String, String>,

    /// This field is specific for Windows operating systems.
    ///
    /// Please read the documentation of [`ToolWindowsMetadata`]
    /// of its purpose.
    #[builder(default)]
    #[serde(default)]
    pub windows: ToolWindowsMetadata,

    /// This field represents download links for a tool across
    /// different operating systems if the tool cannot be installed
    /// using an operating system automatically through a
    /// package manager.
    #[builder(default)]
    #[serde(default)]
    pub downloads: ToolPlatformDownloads,
}

/// Windows-specific metadata on how a tool should run in Windows.
#[derive(Debug, Deserialize, Builder, Clone, Default, PartialEq, Eq, Serialize)]
#[builder(builder_type(vis = "pub(crate)"))]
pub struct ToolWindowsMetadata {
    /// Candidate execution absolute paths of
    /// where the tool should run.
    pub exec_paths: Vec<PathBuf>,
}

/// Represents download links for a tool across different operating systems.
///
/// Each field contains an optional URL pointing to the installer or binary
/// for the corresponding platform. If a platform is not supported, its
/// field can be `None`.
#[derive(Debug, Default, Builder, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ToolPlatformDownloads {
    /// Download URL for Windows, if available.
    pub windows: Option<String>,

    /// Download URL for macOS, if available.
    pub macos: Option<String>,

    /// Download URL for Linux, if available.
    pub linux: Option<String>,
}

#[cfg(test)]
mod tests {
    use crate::registry::Toolkit;

    #[test]
    fn should_load_builtin_toolkit() {
        _ = Toolkit::default();
    }
}
