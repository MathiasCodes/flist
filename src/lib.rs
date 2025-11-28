//! FList - File List with Version Information
//!
//! A fast, cross-platform command-line tool for searching and listing files
//! with optional file version information extraction from PE files.
//!
//! # Features
//!
//! - Recursive file search with pattern matching
//! - Extract and display file version information from PE files (.exe, .dll)
//! - Filter files by version constraints (min/max)
//! - Sort output by file path
//! - Export results to file
//! - Cross-platform support (Windows, Linux, macOS)
//!
//! # Examples
//!
//! ```no_run
//! use std::path::Path;
//! use flist::file_lister::enumerate_files;
//!
//! let files = enumerate_files(Path::new("."), "*.rs").unwrap();
//! println!("Found {} files", files.len());
//! ```

pub mod cli;
pub mod file_lister;
pub mod file_version;
pub mod output;
pub mod version_reader;
