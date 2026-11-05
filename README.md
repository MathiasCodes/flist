# FList - File List with Version Information

A fast, cross-platform command-line tool for searching and listing files with optional file version information extraction from PE files (Windows executables and DLLs).

## Features

- **Recursive file search** with pattern matching (supports wildcards)
- **Extract and display file version information** from PE files (.exe, .dll)
- **Filter files by version constraints** (minimum/maximum version)
- **Sort output** by file path
- **Export results to file** for further processing
- **Fast and efficient** - written in Rust for maximum performance
- **Cross-platform** - works on Windows, Linux, and macOS
- **Robust error handling** - continues on access denied errors

## Installation

### From Binary Releases

Download the latest release for your platform from the [Releases](https://github.com/MathiasCodes/flist/releases) page:

- **Windows**: `flist-windows-x86_64.exe`
- **Linux**: `flist-linux-x86_64`
- **macOS**: `flist-macos-x86_64`

Rename the binary to `flist` (or `flist.exe` on Windows) and add it to your PATH for easy access.

### From Source

Requires Rust 1.85 or later:

```bash
git clone https://github.com/MathiasCodes/flist.git
cd flist
cargo build --release
```

The binary will be available at `target/release/flist` (or `flist.exe` on Windows).

## Usage

### Basic Examples

```bash
# List all files in current directory
flist

# List all DLL files
flist "*.dll"

# List DLL files with version information
flist "*.dll" --file-version

# Short form
flist "*.dll" -f
```

### Version Filtering

```bash
# Filter by minimum version
flist "*.dll" --min-version 10.0

# Filter by maximum version
flist "*.dll" --max-version 11.0.0.0

# Filter by version range and show the versions
flist "*.dll" -f --min-version 10.0 --max-version 11.0
```

### Advanced Usage

```bash
# Search in specific directory
flist "*.exe" -d C:\Windows\System32

# Sort output by file path
flist "*.dll" --sort path
flist "*.dll" -s

# Show versions, newest first
flist "*.dll" -f --sort version --reverse
flist "*.dll" -fvr

# Save output to file (in addition to the console)
flist "*.dll" -f -o output.txt

# Skip inaccessible directories without warnings
flist "*.dll" --ignore-inaccessible

# Combine multiple options
flist "*.dll" -f -s --min-version 10.0 --max-version 10.0.30000.0 -d C:\Windows\System32 -o results.txt
```

## Command-Line Options

| Option | Short | Description |
|--------|-------|-------------|
| `<PATTERN>` | | Search pattern for file names (default: `*`); wildcards `*` and `?`, case-insensitive |
| `--file-version` | `-f` | Include the file version in the output |
| `--sort <KEY>` | | Sort the output by `path` (case-insensitive) or `version` (files without version last) |
| | `-s` | Short form of `--sort path` |
| | `-v` | Short form of `--sort version` |
| `--reverse` | `-r` | Reverse the sort order (requires a sort option) |
| `--min-version <VERSION>` | | Omit files whose version is lower than `<VERSION>` |
| `--max-version <VERSION>` | | Omit files whose version is higher than `<VERSION>` |
| `--version-source <SOURCE>` | | Version to use: `file` (default), `product`, `file-fixed` or `product-fixed` |
| `--dir <DIR>` | `-d` | Directory to search (default: current directory) |
| `--output <FILE>` | `-o` | Additionally write the output to `<FILE>` (created or overwritten) |
| `--ignore-inaccessible` | | Skip inaccessible directories and files silently |
| `--help` | `-h` | Print help information |
| `--version` | `-V` | Print the program version |

Values may be given as `--dir C:\Tools` or `--dir=C:\Tools`; short flags may be combined (`-fvr`).
If several sort options are given, the last one wins.

The version filter works independently of `--file-version`, which only controls whether the version column is shown.
Version bounds are inclusive; missing parts count as `0` (`--max-version 1.2` keeps `1.2.0.0`).
Files without a comparable version are omitted while a version filter is active.

## Output

One line per file, encoded as UTF-8 and terminated by the platform's line ending.
With `--file-version` the version is left-aligned in a column of 16 characters, followed by the full path:

```text
1.1.30.01       C:\Tools\AutoHotkey\BasicTextExpansion.exe
14.08           C:\Tools\Autoruns\autoruns.exe
                C:\Tools\Cheatbox\cheatbox.exe
```

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | Runtime error (e.g. directory not found, output file not writable), or inaccessible entries were skipped with a warning |
| `2` | Usage error (e.g. unknown option, invalid version argument) |

See [SPECIFICATION.md](SPECIFICATION.md) for the complete behaviour.

## Building from Source

### Prerequisites

- Rust 1.85 or later
- Cargo (comes with Rust installation)

### Build Commands

```bash
# Debug build (faster compilation, slower execution)
cargo build

# Release build (optimized for performance)
cargo build --release

# Run tests
cargo test

# Run directly with Cargo
cargo run -- "*.dll" --include-file-version
```

### Cross-Compilation

To build for Windows from Linux:
```bash
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

To build for Linux from Windows:
```bash
rustup target add x86_64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu
```

## Platform Notes

- Versions are read from Windows executables and libraries (PE files) by parsing the file format directly, without Windows APIs.
  The result is the same on every platform, so Windows binaries can be analysed on Linux and macOS as well.
- Native Linux (ELF) and macOS (Mach-O) binaries have no version resource and are listed without a version.
- Lines end with CRLF on Windows and LF on Linux and macOS.

## Version Format

`--version-source` selects which value of the version resource is used for display, filtering and sorting:

| Source | Value | Example |
|--------|-------|---------|
| `file` (default) | `FileVersion` string, as shown by Windows Explorer | `10.0.26100.1 (WinBuild.160101.0800)` |
| `product` | `ProductVersion` string, often the release version of .NET builds | `0.9.4+f67c0dc8fad0` |
| `file-fixed` | Numeric file version | `10.0.26100.7019` |
| `product-fixed` | Numeric product version | `0.9.4.0` |

Strings are shown exactly as stored; numeric versions always have four parts.

For filtering and sorting, the version is interpreted as `major.minor.build.private`.
Each part uses its leading digits; the rest is ignored (`1.0.0-beta` compares as `1.0.0.0`).

## License

MIT License - see [LICENSE](LICENSE) file for details.
