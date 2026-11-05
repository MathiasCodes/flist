//! Command-line argument parsing and configuration.
//!
//! This module defines the command-line interface for FList using the `clap` crate.

use crate::file_version::FileVersion;
use crate::version_reader::VersionSource;
use clap::{ArgGroup, Parser, ValueEnum};
use std::path::PathBuf;

/// The key by which the output is sorted.
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    /// Sort by full path, case-insensitively
    Path,
    /// Sort by file version; files without a comparable version come last
    Version,
}

/// Command-line arguments for FList.
///
/// # Examples
///
/// ```
/// use clap::Parser;
/// use flist::cli::CliArgs;
///
/// let args = CliArgs::parse_from(["flist", "*.dll", "--file-version"]);
/// assert_eq!(args.pattern, "*.dll");
/// assert!(args.file_version);
/// ```
#[derive(Parser, Debug)]
#[command(name = "flist")]
#[command(version)]
#[command(
    about = "Lists files in a directory and its subdirectories, optionally with their file version"
)]
#[command(args_override_self = true)]
#[command(group(ArgGroup::new("sorting").multiple(true)))]
pub struct CliArgs {
    /// Search pattern for file names (wildcards * and ?, case-insensitive)
    #[arg(default_value = "*")]
    pub pattern: String,

    /// Include the file version in the output
    #[arg(short = 'f', long = "file-version")]
    pub file_version: bool,

    /// Sort the output by KEY
    #[arg(
        long = "sort",
        value_name = "KEY",
        group = "sorting",
        overrides_with_all = ["sort_path", "sort_version"]
    )]
    pub sort: Option<SortKey>,

    /// Short form of --sort path
    #[arg(
        short = 's',
        group = "sorting",
        overrides_with_all = ["sort", "sort_version"]
    )]
    pub sort_path: bool,

    /// Short form of --sort version
    #[arg(
        short = 'v',
        group = "sorting",
        overrides_with_all = ["sort", "sort_path"]
    )]
    pub sort_version: bool,

    /// Reverse the sort order (requires a sort option)
    #[arg(short = 'r', long = "reverse", requires = "sorting")]
    pub reverse: bool,

    /// Omit files whose file version is lower than VERSION (e.g. 1.2.3.4)
    #[arg(long = "min-version", value_name = "VERSION")]
    pub min_version: Option<FileVersion>,

    /// Omit files whose file version is higher than VERSION (e.g. 2.0)
    #[arg(long = "max-version", value_name = "VERSION")]
    pub max_version: Option<FileVersion>,

    /// Take the version from SOURCE (shown, filtered and sorted)
    #[arg(
        long = "version-source",
        value_name = "SOURCE",
        value_enum,
        default_value_t = VersionSource::File
    )]
    pub version_source: VersionSource,

    /// Start the search in DIR [default: the current directory]
    #[arg(short = 'd', long = "dir", value_name = "DIR")]
    pub dir: Option<PathBuf>,

    /// Additionally write the output to FILE (created or overwritten)
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Skip inaccessible directories and files silently
    #[arg(long = "ignore-inaccessible")]
    pub ignore_inaccessible: bool,
}

impl CliArgs {
    /// Returns `true` if a version filter is active.
    pub fn has_version_filter(&self) -> bool {
        self.min_version.is_some() || self.max_version.is_some()
    }

    /// Returns the effective sort key; the last of `--sort`, `-s` and `-v` wins.
    pub fn sort_key(&self) -> Option<SortKey> {
        if self.sort_path {
            Some(SortKey::Path)
        } else if self.sort_version {
            Some(SortKey::Version)
        } else {
            self.sort
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_without_arguments_uses_defaults() {
        let args = CliArgs::parse_from(["flist"]);
        assert_eq!(args.pattern, "*");
        assert!(!args.file_version);
        assert_eq!(args.sort_key(), None);
        assert!(!args.reverse);
        assert!(!args.ignore_inaccessible);
        assert!(!args.has_version_filter());
    }

    #[test]
    fn parse_short_flags() {
        let args = CliArgs::parse_from(["flist", "-f", "-s", "-d", "dir", "-o", "out.txt"]);
        assert!(args.file_version);
        assert_eq!(args.sort_key(), Some(SortKey::Path));
        assert_eq!(args.dir, Some(PathBuf::from("dir")));
        assert_eq!(args.output, Some(PathBuf::from("out.txt")));
    }

    #[test]
    fn parse_long_options_with_equals_sign() {
        let args = CliArgs::parse_from([
            "flist",
            "*.exe",
            "--file-version",
            "--sort=version",
            "--reverse",
            "--min-version=1.0",
            "--max-version",
            "2.0.0.0",
            "--dir=C:\\Windows",
            "--output=results.txt",
            "--ignore-inaccessible",
        ]);
        assert_eq!(args.pattern, "*.exe");
        assert!(args.file_version);
        assert_eq!(args.sort_key(), Some(SortKey::Version));
        assert!(args.reverse);
        assert_eq!(args.min_version, Some("1.0".parse().unwrap()));
        assert_eq!(args.max_version, Some("2.0.0.0".parse().unwrap()));
        assert_eq!(args.dir, Some(PathBuf::from("C:\\Windows")));
        assert_eq!(args.output, Some(PathBuf::from("results.txt")));
        assert!(args.ignore_inaccessible);
    }

    #[test]
    fn version_filter_does_not_enable_file_version() {
        let args = CliArgs::parse_from(["flist", "--min-version", "1.0"]);
        assert!(args.has_version_filter());
        assert!(!args.file_version);
    }

    #[test]
    fn sort_value_may_follow_after_space() {
        let args = CliArgs::parse_from(["flist", "--sort", "path", "*.dll"]);
        assert_eq!(args.sort_key(), Some(SortKey::Path));
        assert_eq!(args.pattern, "*.dll");
    }

    #[test]
    fn version_source_defaults_to_file() {
        let args = CliArgs::parse_from(["flist"]);
        assert_eq!(args.version_source, VersionSource::File);
    }

    #[test]
    fn parse_version_sources() {
        let cases = [
            ("file", VersionSource::File),
            ("product", VersionSource::Product),
            ("file-fixed", VersionSource::FileFixed),
            ("product-fixed", VersionSource::ProductFixed),
        ];
        for (value, expected) in cases {
            let args = CliArgs::parse_from(["flist", "--version-source", value]);
            assert_eq!(args.version_source, expected, "value {value:?}");
        }
    }

    #[test]
    fn invalid_version_source_is_usage_error() {
        let err = CliArgs::try_parse_from(["flist", "--version-source", "assembly"]).unwrap_err();
        assert_eq!(err.exit_code(), 2);
    }

    #[test]
    fn last_sort_option_wins() {
        let cases: [(&[&str], SortKey); 5] = [
            (&["-s", "-v"], SortKey::Version),
            (&["-v", "-s"], SortKey::Path),
            (&["--sort", "path", "-v"], SortKey::Version),
            (&["-v", "--sort", "path"], SortKey::Path),
            (&["--sort", "path", "--sort", "version"], SortKey::Version),
        ];
        for (options, expected) in cases {
            let args = CliArgs::parse_from(std::iter::once(&"flist").chain(options));
            assert_eq!(args.sort_key(), Some(expected), "options {options:?}");
        }
    }

    #[test]
    fn short_flags_can_be_combined() {
        let args = CliArgs::parse_from(["flist", "-fvr"]);
        assert!(args.file_version);
        assert_eq!(args.sort_key(), Some(SortKey::Version));
        assert!(args.reverse);
    }

    #[test]
    fn reverse_without_sort_is_usage_error() {
        let err = CliArgs::try_parse_from(["flist", "-r"]).unwrap_err();
        assert_eq!(err.exit_code(), 2);
    }

    #[test]
    fn invalid_or_missing_sort_key_is_usage_error() {
        for options in [&["--sort", "size"][..], &["--sort"][..]] {
            let err =
                CliArgs::try_parse_from(std::iter::once(&"flist").chain(options)).unwrap_err();
            assert_eq!(err.exit_code(), 2, "options {options:?}");
        }
    }

    #[test]
    fn unknown_option_is_usage_error() {
        let err = CliArgs::try_parse_from(["flist", "-x"]).unwrap_err();
        assert_eq!(err.exit_code(), 2);
    }

    #[test]
    fn second_pattern_is_usage_error() {
        let err = CliArgs::try_parse_from(["flist", "*.dll", "*.exe"]).unwrap_err();
        assert_eq!(err.exit_code(), 2);
    }

    #[test]
    fn invalid_version_argument_is_usage_error() {
        for value in ["abc", "", "1.0-beta", "1.2.3.4.5"] {
            let err = CliArgs::try_parse_from(["flist", "--min-version", value]).unwrap_err();
            assert_eq!(err.exit_code(), 2, "value {value:?}");
        }
    }
}
