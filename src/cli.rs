//! Command-line argument parsing and configuration.
//!
//! This module defines the command-line interface for FList using the `clap` crate.

use clap::Parser;

/// Command-line arguments for FList.
///
/// FList is a tool for listing files in directories with optional file version information
/// extraction from PE files (Windows executables and DLLs).
///
/// # Examples
///
/// ```no_run
/// use clap::Parser;
/// use flist::cli::CliArgs;
///
/// let args = CliArgs::parse();
/// println!("Pattern: {}", args.pattern);
/// ```
#[derive(Parser, Debug)]
#[command(name = "flist")]
#[command(version)]
#[command(about = "List files in directories with optional file version information", long_about = None)]
pub struct CliArgs {
    /// Search pattern (e.g., *.dll, *.exe)
    #[arg(default_value = "*")]
    pub pattern: String,

    /// Include file version information
    #[arg(short = 'i', long = "ifs")]
    pub include_file_version: bool,

    /// Sort output by file path
    #[arg(short = 's', long = "sp")]
    pub sort_by_path: bool,

    /// Minimum version filter (e.g., 1.2.3.4)
    /// Format: -minv:1.2.3.4 or --minv 1.2.3.4
    #[arg(long = "minv", value_name = "VERSION")]
    pub min_version: Option<String>,

    /// Maximum version filter (e.g., 2.0.0.0)
    /// Format: -maxv:2.0.0.0 or --maxv 2.0.0.0
    #[arg(long = "maxv", value_name = "VERSION")]
    pub max_version: Option<String>,

    /// Working directory to search
    /// Format: -d:C:\path or --directory C:\path
    #[arg(short = 'd', long = "directory", value_name = "PATH")]
    pub directory: Option<String>,

    /// Output file path (in addition to console output)
    /// Format: -o:output.txt or --output output.txt
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    pub output_file: Option<String>,

    /// Quiet mode - only show results
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,
}

impl CliArgs {
    /// Normalizes the command-line arguments.
    ///
    /// Auto-enables `include_file_version` if `min_version` or `max_version` is specified,
    /// since version filtering requires version information to be extracted.
    ///
    /// # Examples
    ///
    /// ```
    /// use flist::cli::CliArgs;
    /// use clap::Parser;
    ///
    /// let mut args = CliArgs::parse_from(&["flist", "--minv", "1.0.0.0"]);
    /// args.normalize();
    /// assert!(args.include_file_version);
    /// ```
    pub fn normalize(&mut self) {
        if self.min_version.is_some() || self.max_version.is_some() {
            self.include_file_version = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_pattern() {
        let args = CliArgs::parse_from(&["flist"]);
        assert_eq!(args.pattern, "*");
        assert!(!args.include_file_version);
        assert!(!args.sort_by_path);
        assert!(!args.quiet);
    }

    #[test]
    fn test_custom_pattern() {
        let args = CliArgs::parse_from(&["flist", "*.dll"]);
        assert_eq!(args.pattern, "*.dll");
    }

    #[test]
    fn test_flags() {
        let args = CliArgs::parse_from(&["flist", "-i", "-s", "-q"]);
        assert!(args.include_file_version);
        assert!(args.sort_by_path);
        assert!(args.quiet);
    }

    #[test]
    fn test_long_flags() {
        let args = CliArgs::parse_from(&["flist", "--ifs", "--sp", "--quiet"]);
        assert!(args.include_file_version);
        assert!(args.sort_by_path);
        assert!(args.quiet);
    }

    #[test]
    fn test_version_filters() {
        let args = CliArgs::parse_from(&["flist", "--minv", "1.0.0.0", "--maxv", "2.0.0.0"]);
        assert_eq!(args.min_version, Some("1.0.0.0".to_string()));
        assert_eq!(args.max_version, Some("2.0.0.0".to_string()));
    }

    #[test]
    fn test_directory_and_output() {
        let args = CliArgs::parse_from(&["flist", "-d", "C:\\test", "-o", "output.txt"]);
        assert_eq!(args.directory, Some("C:\\test".to_string()));
        assert_eq!(args.output_file, Some("output.txt".to_string()));
    }

    #[test]
    fn test_normalize_auto_enables_version() {
        let mut args = CliArgs::parse_from(&["flist", "--minv", "1.0.0.0"]);
        assert!(!args.include_file_version); // Not set initially
        args.normalize();
        assert!(args.include_file_version); // Auto-enabled
    }

    #[test]
    fn test_all_options_combined() {
        let args = CliArgs::parse_from(&[
            "flist",
            "*.exe",
            "-i",
            "-s",
            "-q",
            "--minv",
            "1.0.0.0",
            "--maxv",
            "2.0.0.0",
            "-d",
            "C:\\Windows",
            "-o",
            "results.txt",
        ]);
        assert_eq!(args.pattern, "*.exe");
        assert!(args.include_file_version);
        assert!(args.sort_by_path);
        assert!(args.quiet);
        assert_eq!(args.min_version, Some("1.0.0.0".to_string()));
        assert_eq!(args.max_version, Some("2.0.0.0".to_string()));
        assert_eq!(args.directory, Some("C:\\Windows".to_string()));
        assert_eq!(args.output_file, Some("results.txt".to_string()));
    }
}
