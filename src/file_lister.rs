//! File enumeration, filtering, and sorting functionality.
//!
//! This module provides functions for recursively searching directories for files
//! matching a pattern, collecting file information with optional version extraction,
//! filtering by version constraints, and sorting results.

use crate::file_version::FileVersion;
use crate::version_reader::read_file_version;
use glob::Pattern;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Information about a file including its path and optional version.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use flist::file_lister::FileInfo;
///
/// let info = FileInfo {
///     path: PathBuf::from("test.dll"),
///     version: None,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub version: Option<FileVersion>,
}

/// Recursively enumerates files matching the search pattern.
///
/// Walks the directory tree starting from `directory` and returns all files
/// whose names match the glob pattern.
///
/// # Arguments
///
/// * `directory` - Root directory to start searching from
/// * `pattern` - Glob pattern to match file names (e.g., "*.dll", "kernel*.exe")
///
/// # Returns
///
/// A vector of paths to files matching the pattern, or an error if the pattern
/// is invalid or directory cannot be accessed.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use flist::file_lister::enumerate_files;
///
/// let files = enumerate_files(Path::new("."), "*.rs").unwrap();
/// for file in files {
///     println!("{}", file.display());
/// }
/// ```
pub fn enumerate_files(directory: &Path, pattern: &str) -> Result<Vec<PathBuf>, anyhow::Error> {
    let glob_pattern = Pattern::new(pattern)?;

    let files: Vec<PathBuf> = WalkDir::new(directory)
        .into_iter()
        .filter_map(|e| e.ok()) // Skip entries with errors (permission denied, etc.)
        .filter(|e| e.file_type().is_file()) // Only files, not directories
        .filter(|e| {
            // Match file name against glob pattern
            e.file_name()
                .to_str()
                .map(|name| glob_pattern.matches(name))
                .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    Ok(files)
}

/// Collects file information with optional version reading.
///
/// Takes a list of file paths and creates `FileInfo` structures, optionally
/// extracting version information from each file.
///
/// # Arguments
///
/// * `files` - Vector of file paths to process
/// * `include_version` - Whether to extract version information from files
///
/// # Returns
///
/// A vector of `FileInfo` structures containing paths and optional versions.
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
/// use flist::file_lister::collect_file_info;
///
/// let files = vec![PathBuf::from("test.dll")];
/// let info = collect_file_info(files, false);
/// ```
pub fn collect_file_info(files: Vec<PathBuf>, include_version: bool) -> Vec<FileInfo> {
    files
        .into_iter()
        .map(|path| {
            let version = if include_version {
                // Try to read version, but don't fail if it's not available
                read_file_version(&path).ok().flatten()
            } else {
                None
            };
            FileInfo { path, version }
        })
        .collect()
}

/// Filters files by version constraints.
///
/// Keeps only files whose versions fall within the specified range.
/// Files without version information are excluded.
///
/// # Arguments
///
/// * `files` - Vector of file information to filter
/// * `min_version` - Minimum version (inclusive), or None for no minimum
/// * `max_version` - Maximum version (inclusive), or None for no maximum
///
/// # Returns
///
/// A filtered vector containing only files matching the version constraints.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use std::str::FromStr;
/// use flist::file_lister::{FileInfo, filter_by_version};
/// use flist::file_version::FileVersion;
///
/// let files = vec![
///     FileInfo {
///         path: PathBuf::from("test.dll"),
///         version: Some(FileVersion::from_str("1.5.0.0").unwrap()),
///     },
/// ];
/// let min = Some(FileVersion::from_str("1.0.0.0").unwrap());
/// let max = Some(FileVersion::from_str("2.0.0.0").unwrap());
/// let filtered = filter_by_version(files, min, max);
/// assert_eq!(filtered.len(), 1);
/// ```
pub fn filter_by_version(
    files: Vec<FileInfo>,
    min_version: Option<FileVersion>,
    max_version: Option<FileVersion>,
) -> Vec<FileInfo> {
    files
        .into_iter()
        .filter(|file_info| {
            if let Some(version) = file_info.version {
                let min_ok = min_version.is_none_or(|min| version >= min);
                let max_ok = max_version.is_none_or(|max| version <= max);
                min_ok && max_ok
            } else {
                // Files without version info are excluded when filtering by version
                false
            }
        })
        .collect()
}

/// Sorts files by path in ascending order.
///
/// # Arguments
///
/// * `files` - Vector of file information to sort
///
/// # Returns
///
/// The same vector sorted by file path.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use flist::file_lister::{FileInfo, sort_by_path};
///
/// let mut files = vec![
///     FileInfo { path: PathBuf::from("z.dll"), version: None },
///     FileInfo { path: PathBuf::from("a.dll"), version: None },
/// ];
/// let sorted = sort_by_path(files);
/// assert_eq!(sorted[0].path, PathBuf::from("a.dll"));
/// ```
pub fn sort_by_path(mut files: Vec<FileInfo>) -> Vec<FileInfo> {
    files.sort_by(|a, b| a.path.cmp(&b.path));
    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_enumerate_files_with_pattern() {
        // Create a temporary directory with test files
        let temp_dir = std::env::temp_dir().join("flist_test_enum");
        let _ = fs::remove_dir_all(&temp_dir); // Clean up if exists
        fs::create_dir_all(&temp_dir).unwrap();

        // Create test files
        fs::File::create(temp_dir.join("test1.txt")).unwrap();
        fs::File::create(temp_dir.join("test2.txt")).unwrap();
        fs::File::create(temp_dir.join("test.dll")).unwrap();
        fs::File::create(temp_dir.join("other.exe")).unwrap();

        // Test pattern matching
        let files = enumerate_files(&temp_dir, "*.txt").unwrap();
        assert_eq!(files.len(), 2);

        let files = enumerate_files(&temp_dir, "*.dll").unwrap();
        assert_eq!(files.len(), 1);

        let files = enumerate_files(&temp_dir, "*").unwrap();
        assert_eq!(files.len(), 4);

        // Clean up
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_collect_file_info_without_version() {
        let paths = vec![PathBuf::from("test1.txt"), PathBuf::from("test2.txt")];

        let file_infos = collect_file_info(paths, false);
        assert_eq!(file_infos.len(), 2);
        assert!(file_infos[0].version.is_none());
        assert!(file_infos[1].version.is_none());
    }

    #[test]
    fn test_filter_by_version() {
        let files = vec![
            FileInfo {
                path: PathBuf::from("file1.dll"),
                version: Some("1.0.0.0".parse().unwrap()),
            },
            FileInfo {
                path: PathBuf::from("file2.dll"),
                version: Some("2.0.0.0".parse().unwrap()),
            },
            FileInfo {
                path: PathBuf::from("file3.dll"),
                version: Some("3.0.0.0".parse().unwrap()),
            },
            FileInfo {
                path: PathBuf::from("file4.dll"),
                version: None,
            },
        ];

        // Filter with min version
        let filtered = filter_by_version(files.clone(), Some("2.0.0.0".parse().unwrap()), None);
        assert_eq!(filtered.len(), 2); // 2.0.0.0 and 3.0.0.0

        // Filter with max version
        let filtered = filter_by_version(files.clone(), None, Some("2.0.0.0".parse().unwrap()));
        assert_eq!(filtered.len(), 2); // 1.0.0.0 and 2.0.0.0

        // Filter with both min and max
        let filtered = filter_by_version(
            files.clone(),
            Some("1.5.0.0".parse().unwrap()),
            Some("2.5.0.0".parse().unwrap()),
        );
        assert_eq!(filtered.len(), 1); // Only 2.0.0.0

        // Files without version are excluded
        let filtered = filter_by_version(files.clone(), Some("0.0.0.0".parse().unwrap()), None);
        assert_eq!(filtered.len(), 3); // file4.dll is excluded
    }

    #[test]
    fn test_sort_by_path() {
        let files = vec![
            FileInfo {
                path: PathBuf::from("c.txt"),
                version: None,
            },
            FileInfo {
                path: PathBuf::from("a.txt"),
                version: None,
            },
            FileInfo {
                path: PathBuf::from("b.txt"),
                version: None,
            },
        ];

        let sorted = sort_by_path(files);
        assert_eq!(sorted[0].path, PathBuf::from("a.txt"));
        assert_eq!(sorted[1].path, PathBuf::from("b.txt"));
        assert_eq!(sorted[2].path, PathBuf::from("c.txt"));
    }

    #[test]
    fn test_enumerate_files_recursive() {
        // Create a temporary directory with subdirectories
        let temp_dir = std::env::temp_dir().join("flist_test_recursive");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir.join("subdir1")).unwrap();
        fs::create_dir_all(&temp_dir.join("subdir2")).unwrap();

        // Create test files in different directories
        fs::File::create(temp_dir.join("root.txt")).unwrap();
        fs::File::create(temp_dir.join("subdir1").join("sub1.txt")).unwrap();
        fs::File::create(temp_dir.join("subdir2").join("sub2.txt")).unwrap();

        // Test recursive enumeration
        let files = enumerate_files(&temp_dir, "*.txt").unwrap();
        assert_eq!(files.len(), 3);

        // Clean up
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_invalid_pattern() {
        let temp_dir = std::env::temp_dir();
        // Invalid glob pattern with unclosed bracket
        let result = enumerate_files(&temp_dir, "[invalid");
        assert!(result.is_err());
    }
}
