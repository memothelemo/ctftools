use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::LazyLock;
use tracing::{debug, trace};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolMetadata {
    pub name: String,
    pub command: String,
    pub description: String,
    pub packages: HashMap<String, String>,
    pub windows: ToolWindowsMetadata,
    pub download_links: ToolDownloadLinks,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
pub struct ToolWindowsMetadata {
    pub exec_paths: Vec<PathBuf>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
pub struct ToolDownloadLinks {
    pub windows: Option<String>,
    pub macos: Option<String>,
    pub linux: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawToolInfo {
    pub name: String,
    pub description: String,

    #[serde(default)]
    pub packages: HashMap<String, String>,

    #[serde(default)]
    pub windows: ToolWindowsMetadata,

    #[serde(default)]
    pub download_links: ToolDownloadLinks,
}

pub static BUILTIN_TOOLS: LazyLock<Vec<ToolMetadata>> = LazyLock::new(|| {
    // deserialize the built-in json file
    let map: BTreeMap<String, RawToolInfo> = toml::from_str(include_str!("builtin-tools.toml"))
        .expect("failed to deserialize builtin tools");

    // if name field is empty, we can override with its associated key
    let mut tools = Vec::new();
    for (command, mut tool) in map.into_iter() {
        trace!(?tool, "found builtin tool");

        let name = if tool.name.is_empty() {
            command.clone()
        } else {
            tool.name
        };

        // use the command key as a package if 'default' key is missing
        tool.packages
            .entry("default".to_string())
            .or_insert_with(|| command.clone());

        tool.description = tool.description.trim().to_string();
        tools.push(ToolMetadata {
            name,
            command,
            description: tool.description,
            packages: tool.packages,
            windows: tool.windows,
            download_links: tool.download_links,
        });
    }

    debug!(tools.len = %tools.len(), "successfully loaded builtin tools registry");
    tools
});
