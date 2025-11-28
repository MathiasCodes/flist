# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
- Migrated from .NET 6.0 to Rust 2021 edition
- Improved file enumeration performance (23% faster than .NET version)
- Enhanced error messages with helpful context and expected formats
- Better cross-platform support for PE file analysis

### Technical Details
- Uses clap 4.5 for command-line argument parsing
- Uses walkdir for efficient recursive directory traversal
- Uses glob for pattern matching
- Uses pelite for PE file parsing
- Uses anyhow for ergonomic error handling
- Optimized release profile with LTO and size optimization

## [0.x.x] - Previous .NET versions
- See original .NET repository for history

