//! Output formatting for console and file output.
//!
//! This module provides functions for displaying file listing results to the
//! console and writing them to output files.

use crate::file_lister::FileInfo;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Prints results to the console.
///
/// Displays file information with optional version numbers. In non-quiet mode,
/// also shows a summary of the number of files found.
///
/// # Arguments
///
/// * `files` - Slice of file information to display
/// * `include_version` - Whether to display version information
/// * `quiet` - Whether to suppress summary messages
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use flist::file_lister::FileInfo;
/// use flist::output::print_results;
///
/// let files = vec![
///     FileInfo { path: PathBuf::from("test.dll"), version: None },
/// ];
/// print_results(&files, false, true);
/// ```
pub fn print_results(files: &[FileInfo], include_version: bool, quiet: bool) {
    if !quiet {
        println!("Found {} files.", files.len());
        println!();
    }

    for file_info in files {
        if include_version {
            if let Some(version) = file_info.version {
                println!("{:<15} {}", version, file_info.path.display());
            } else {
                println!("{:<15} {}", "", file_info.path.display());
            }
        } else {
            println!("{}", file_info.path.display());
        }
    }

    if !quiet {
        println!();
        println!("Found {} files.", files.len());
    }
}

/// Writes results to a file.
///
/// Creates or overwrites the specified file with the file listing results.
/// Each line contains the file path and optionally the version information.
///
/// # Arguments
///
/// * `files` - Slice of file information to write
/// * `output_path` - Path to the output file
/// * `include_version` - Whether to include version information
///
/// # Returns
///
/// `Ok(())` on success, or an error if the file cannot be created or written.
///
/// # Examples
///
/// ```no_run
/// use std::path::{Path, PathBuf};
/// use flist::file_lister::FileInfo;
/// use flist::output::write_to_file;
///
/// let files = vec![
///     FileInfo { path: PathBuf::from("test.dll"), version: None },
/// ];
/// write_to_file(&files, Path::new("output.txt"), false).unwrap();
/// ```
pub fn write_to_file(
    files: &[FileInfo],
    output_path: &Path,
    include_version: bool,
) -> Result<(), anyhow::Error> {
    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);

    for file_info in files {
        if include_version {
            if let Some(version) = file_info.version {
                writeln!(writer, "{:<15} {}", version, file_info.path.display())?;
            } else {
                writeln!(writer, "{:<15} {}", "", file_info.path.display())?;
            }
        } else {
            writeln!(writer, "{}", file_info.path.display())?;
        }
    }

    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_version::FileVersion;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_write_to_file_without_version() {
        let files = vec![
            FileInfo {
                path: PathBuf::from("file1.txt"),
                version: None,
            },
            FileInfo {
                path: PathBuf::from("file2.txt"),
                version: None,
            },
        ];

        let temp_file = std::env::temp_dir().join("flist_test_output.txt");
        let result = write_to_file(&files, &temp_file, false);
        assert!(result.is_ok());

        let content = fs::read_to_string(&temp_file).unwrap();
        assert!(content.contains("file1.txt"));
        assert!(content.contains("file2.txt"));

        fs::remove_file(&temp_file).unwrap();
    }

    #[test]
    fn test_write_to_file_with_version() {
        let files = vec![
            FileInfo {
                path: PathBuf::from("file1.dll"),
                version: Some("1.0.0.0".parse::<FileVersion>().unwrap()),
            },
            FileInfo {
                path: PathBuf::from("file2.dll"),
                version: Some("2.0.0.0".parse::<FileVersion>().unwrap()),
            },
        ];

        let temp_file = std::env::temp_dir().join("flist_test_output_version.txt");
        let result = write_to_file(&files, &temp_file, true);
        assert!(result.is_ok());

        let content = fs::read_to_string(&temp_file).unwrap();
        assert!(content.contains("1.0.0.0"));
        assert!(content.contains("2.0.0.0"));
        assert!(content.contains("file1.dll"));
        assert!(content.contains("file2.dll"));

        fs::remove_file(&temp_file).unwrap();
    }

    #[test]
    fn test_write_to_file_with_mixed_versions() {
        let files = vec![
            FileInfo {
                path: PathBuf::from("file1.dll"),
                version: Some("1.0.0.0".parse::<FileVersion>().unwrap()),
            },
            FileInfo {
                path: PathBuf::from("file2.dll"),
                version: None, // No version
            },
        ];

        let temp_file = std::env::temp_dir().join("flist_test_output_mixed.txt");
        let result = write_to_file(&files, &temp_file, true);
        assert!(result.is_ok());

        let content = fs::read_to_string(&temp_file).unwrap();
        assert!(content.contains("1.0.0.0"));
        assert!(content.contains("file1.dll"));
        assert!(content.contains("file2.dll"));

        fs::remove_file(&temp_file).unwrap();
    }
}
