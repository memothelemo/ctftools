<div align="center">
    <h1 align="center">
        memothelemo's CTF Tools Toolkit
    </h1>
    <a href="https://github.com/memothelemo/ctftools/blob/master/LICENSE.txt">
        <img src="https://img.shields.io/badge/license-GPLv3-blue.svg" alt="License"/>
    </a>
</div>

> [!WARNING]
>
> _This toolkit is **strictly intended for legitimate CTF competitions and educational cybersecurity purposes**._
> _Users must ensure that **they have explicit and proper authorization** before using these tools on any system._
>
> **Use this software at your own risk!**

A CLI utility designed to help participants in Capture The Flag (CTF) competitions quickly browse, reference, and check their required security toolkit.
This ensures that users are prepared for various CTF challenge categories with a [provided list of essential CTF tools](assets/default/toolkit.yml) defined by the author.

**This CLI utility provides**:
- **Interactive Interface** - Allows users to quickly search tools they need and use arrow keys to navigate and select tools.
- **Tool Verification** - Allows users to check which tools from the provided list are currently installed in their system.

## Example Usage
```
$ ctftools
CTF Tool Selector (https://github.com/memothelemo/ctftools)
-----------------------------------------------------------
Choose a tool to see quick usage notes.
Press up or down arrow keys to select a choice

> █
▸ 🔨 tool1
  🔨 tool2
  🔨 tool3
  🔎 Check which tools are installed
  📦 Install missing tools (coming soon!)
  🚪 Exit
```

## Installation

### From GitHub Releases (Recommended)
You can download the latest prebuilt binaries for Windows, Linux and macOS from our [releases page](https://github.com/memothelemo/ctftools/releases). Be sure to read the [System Compatibility](#system-compatibility) section first.

### From Source (Rust)
If you have the Rust toolchain installed, you can build the project from its source code.

1.  Clone the repository:
    ```sh
    git clone https://github.com/memothelemo/ctftools.git
    cd ctftools
    ```

2.  Build the project
    ```sh
    cargo build --release
    ```

3.  The executable binary will be located at `target/release/ctftools`.

## System Compatibility
`ctftools` utility is currently supported for **x86-64 (64-bit) systems** with the following platforms:

- Windows (Windows 11 is fully tested)
- macOS
- Linux

## License
This project is open sourced and licensed under the [**GNU General Public License v3.0**](LICENSE.txt).
