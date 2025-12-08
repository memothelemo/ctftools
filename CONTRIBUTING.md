# Contributing for `ctftools` project
Thank you for your interest in contributing to the `ctftools` utilty tool!

*These are chapters that will provide you with instructions on how to:*
- [Setup the development environment](#setting-up-the-development-environment)
- [Understand the codebase](#codebase)
- [Submit your changes](#submitting-your-changes)

## Setting up the development environment
To get started with `ctftools` development, you need to setup the local
environment to replicate the setup of author's environment when working
with `ctftools` utility tool.

1. **Install the Rust toolchain**
   
   The project is built with Rust toolchain in stable channel, preferably version `1.91.0` or higher.

2. **Clone the source code**

    Get a local copy of the code by cloning the repository from GitHub.
    ```sh
    # Clone with HTTPS
    $ git clone https://github.com/memothelemo/ctftools.git

    # Or, clone with SSH
    $ git clone git@github.com:memothelemo/ctftools.git

    # Change into the project directory
    $ cd ctftools
    ```

3.  **Build the Project**

    You can compile the project using Cargo, Rust's build tool. This command will download dependencies and build the executable.
    ```sh
    $ cargo build
    ```

4.  **Enable Debug Logging Mode**

    You can enable debug logging mode to see the entire logs of how `ctftools` is working on by assigning the environment variable, `CTFTOOLS_DEBUG` to `1`.

    ```sh
    # This part is only applicable for shell-like consoles.
    # |______________|
    $ CTFTOOLS_DEBUG=1 ctftools
    ```

    **Usage**:
    ```
    [INFO ] ctftools::cli:151 - debug logging is enabled
    [DEBUG] ctftools::cli:30 - using environment: LiveEnvironment
    [DEBUG] ctftools::registry::toolkit:80 - found built-in tool: ToolMetadata { name: "Binwalk", command: "binwalk", examples: ["binwalk file.bin", "binwalk -e file.bin"], description: "Analyzes, identifies, and extracts files embedded within binary/firmware images. Essential for carving data out of corrupted or container files.", windows: ToolWindowsMetadata { exec_paths: ["C:\\ProgramData\\chocolatey\\bin\\binwalk.exe"] } }
    [DEBUG] ctftools::registry::toolkit:80 - found built-in tool: ToolMetadata { name: "Burp Suite", command: "burpsuite", examples: [], description: "An integrated platform for testing web applications. Used to proxy, intercept, view, and modify HTTP/S traffic, discover endpoints, and launch attacks.\nThis application utilizes GUI, so please launch it manually.", windows: ToolWindowsMetadata { exec_paths: ["C:\\Program Files\\BurpSuiteCommunity\\BurpSuiteCommunity.exe", "%LOCALAPPDATA%\\Programs\\BurpSuiteCommunity\\BurpSuiteCommunity.exe"] } }
    ```

## Codebase
The `ctftools` project is structured into several modules, each with a specific responsibility. Understanding this structure will help you navigate the code and find the right place for your contributions.

**Here's a breakdown of the main directories inside `src/`**:

- **`cli`**: This module contains all the logic related to the command-line interface.
  - `action.rs`: Defines the `Action` enum, which represents all possible user actions (e.g., checking for tools, installing tools).
  - `interactive.rs`: Implements the interactive fuzzy-finder menu that users see when they run the application without arguments.
  - `options.rs`: Defines the command-line arguments and options using `clap`.

- **`env`**: This is a crucial part of the architecture. It provides an abstraction over the host system's environment through the `Environment` trait.
  - `live.rs`: The "real" implementation that interacts with the live file system and system commands.
  - `mock.rs`: A mock implementation used for testing, allowing us to simulate different system states without affecting the actual system.

- **`install`**: This module handles the logic for installing tools.
  - `task.rs`: Defines `InstallTask`, which represents a concrete plan for installing a tool (e.g., via a package manager, AUR helper, or direct download).

- **`process`**: Provides a robust `ProcessBuilder` for creating and running external commands. This is used to execute package managers or other scripts.

- **`registry`**: Manages the predefined compiled list of CTF tools that `ctftools` knows about. It is responsible for loading the toolkit definition (likely from an embedded file).

- **`util`**: A collection of utility functions used across the application, such as checking for elevated privileges or finding executables.

## Submitting your changes
1. **Fork and Branch**: Fork the repository and create a new branch for your feature or bug fix.
    ```sh
    $ git checkout -b my-awesome-feature
    ```

2. **Code and Test**: Make your changes. If you're adding new functionality, please try to add tests for it. Make sure existing tests still pass.

3. **Format and Lint**: Ensure your code is well-formatted and free of linter warnings.
    ```sh
    $ cargo fmt
    $ cargo clippy -- -D warnings
    ```

4. **Pull Request**: Open a pull request against the `master` branch. Provide a clear title and description of your changes.
