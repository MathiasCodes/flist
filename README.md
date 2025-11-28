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
flist "*.dll" --include-file-version

# Short form
flist "*.dll" -i
```

### Version Filtering

```bash
# Filter by minimum version
flist "*.dll" --minv 10.0.0.0

# Filter by maximum version
flist "*.dll" --maxv 11.0.0.0

# Filter by version range
flist "*.dll" --minv 10.0.0.0 --maxv 11.0.0.0
```

### Advanced Usage

```bash
# Search in specific directory
flist "*.exe" -d C:\Windows\System32

# Sort output by file path
flist "*.dll" --sort-path

# Quiet mode (only show results, no headers)
flist "*.dll" -q

# Save output to file
flist "*.dll" -i -o output.txt

# Combine multiple options
flist "*.dll" -i -s --minv 10.0.0.0 --maxv 10.0.30000.0 -d C:\Windows\System32 -o results.txt
```

## Command-Line Options

| Option | Short | Description |
|--------|-------|-------------|
| `<PATTERN>` | | Search pattern (default: `*`) - supports wildcards |
| `--include-file-version` | `-i` | Include file version information in output |
| `--sort-path` | `-s` | Sort output alphabetically by file path |
| `--minv <VERSION>` | | Omit files with version lower than specified |
| `--maxv <VERSION>` | | Omit files with version higher than specified |
| `--directory <PATH>` | `-d` | Directory to search (default: current directory) |
| `--output <FILE>` | `-o` | Write output to specified file |
| `--quiet` | `-q` | Quiet mode - suppress header and footer text |
| `--help` | `-h` | Print help information |

**Note:** When `--minv` or `--maxv` is specified, `--include-file-version` is automatically enabled.

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

### Windows
- Full PE file version extraction support using native Windows APIs
- Optimal performance with direct API access
- Handles access denied errors gracefully

### Linux/macOS
- Can read PE file version information from Windows binaries
- Useful for analyzing Windows executables on non-Windows platforms
- Uses the same pelite library for cross-platform PE parsing

## Version Format

File versions follow the format: `major.minor.build.private`

Example: `10.0.26100.7019`

- **Major**: Major version number
- **Minor**: Minor version number  
- **Build**: Build number
- **Private**: Private part number

## License

MIT License - see [LICENSE](LICENSE) file for details.
