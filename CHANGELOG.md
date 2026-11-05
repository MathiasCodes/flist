# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2026-11-05

Aligned with [SPECIFICATION.md](SPECIFICATION.md).

### Changed
- **Breaking:** GNU-style options `-f/--file-version`, `--sort <KEY>` with `-s`, `--min-version`, `--max-version`, `-d/--dir`; removed `--ifs`, `--sp`, `--minv`, `--maxv`, `--directory` and `-q/--quiet`
- Output contains only the result lines, without header and footer
- Lines end with CRLF on Windows and LF elsewhere; output is UTF-8
- Without sorting, files of a directory are listed before its subdirectories
- The version is by default the `FileVersion` string as stored in the file instead of the numeric version
- The version filter no longer enables the version column
- Version comparison treats missing parts as `0` and uses the leading digits of each part
- Search pattern uses Windows wildcards (`*`, `?`) and is case-insensitive
- Sorting by path is case-insensitive

### Added
- Sorting by file version with `--sort version` or `-v`
- `-r/--reverse` to reverse the sort order
- `--version-source` to use the product version or the numeric versions instead of the `FileVersion` string
- Warnings for skipped inaccessible entries and `--ignore-inaccessible`
- Exit codes: 1 for runtime errors and skipped entries, 2 for usage errors

### Fixed
- A missing start directory is reported as an error instead of producing an empty list
- `--max-version 1.2` no longer omits files with version `1.2.0.0`
- Empty version arguments are rejected

### Removed
- Unused dependencies `glob` and `winapi`

## [0.9.0] - 2025-11-28

### Added
- Initial Rust implementation of FList
- Recursive file search with glob pattern matching
- PE file version extraction on Windows using pelite crate
- Cross-platform PE file reading on Linux/macOS
- Version filtering with --minv and --maxv options
- Sorting by file path with -s/--sp flag
- Output to file with -o/--output option
- Quiet mode for scripting with -q/--quiet flag
- Comprehensive test suite with 42 tests (29 unit tests + 13 doc tests)
- CI/CD with GitHub Actions for automated testing and builds
- Cross-platform binary releases (Windows, Linux, macOS)
- Comprehensive documentation with README.md and rustdoc comments
- MIT License
- Binary size optimization (388 KB release binary)

### Changed
- Enhanced error messages with helpful context and expected formats
- Better cross-platform support for PE file analysis

### Technical Details
- Uses clap 4.5 for command-line argument parsing
- Uses walkdir for efficient recursive directory traversal
- Uses glob for pattern matching
- Uses pelite for PE file parsing
- Uses anyhow for ergonomic error handling
- Optimized release profile with LTO and size optimization
