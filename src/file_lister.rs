//! File enumeration, pattern matching, filtering and sorting.

use crate::file_version::FileVersion;
use crate::version_reader::read_file_version;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Information about a listed file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileInfo {
    pub path: PathBuf,
    /// The `FileVersion` string as stored in the file, if any.
    pub version: Option<String>,
}

/// The result of a search: the files found and warnings about skipped entries.
#[derive(Debug)]
pub struct SearchResult<T> {
    pub items: Vec<T>,
    pub warnings: Vec<String>,
}

impl<T> Default for SearchResult<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

/// A file name pattern with Windows wildcard semantics, matched case-insensitively.
///
/// `*` matches any number of characters, `?` matches exactly one character.
///
/// # Examples
///
/// ```
/// use flist::file_lister::WildcardPattern;
///
/// let pattern = WildcardPattern::new("*.PDB");
/// assert!(pattern.matches("flist.pdb"));
/// assert!(!pattern.matches("flist.exe"));
/// ```
#[derive(Debug, Clone)]
pub struct WildcardPattern {
    chars: Vec<char>,
}

impl WildcardPattern {
    pub fn new(pattern: &str) -> Self {
        Self {
            chars: fold_case(pattern),
        }
    }

    pub fn matches(&self, name: &str) -> bool {
        let name = fold_case(name);
        let pattern = &self.chars;
        let (mut p, mut n) = (0, 0);
        let mut backtrack: Option<(usize, usize)> = None;

        while n < name.len() {
            if p < pattern.len() && (pattern[p] == '?' || pattern[p] == name[n]) {
                p += 1;
                n += 1;
            } else if p < pattern.len() && pattern[p] == '*' {
                backtrack = Some((p, n));
                p += 1;
            } else if let Some((star, matched)) = backtrack {
                p = star + 1;
                n = matched + 1;
                backtrack = Some((star, n));
            } else {
                return false;
            }
        }
        pattern[p..].iter().all(|&c| c == '*')
    }
}

fn fold_case(s: &str) -> Vec<char> {
    s.chars().flat_map(char::to_lowercase).collect()
}

/// Recursively enumerates files in `directory` whose names match `pattern`.
///
/// The search is depth-first: within each directory, its files come first, then its
/// subdirectories one after another, each with its complete subtree.
/// Files and subdirectories otherwise keep the file system enumeration order.
///
/// Inaccessible entries below the start directory are skipped and reported as warnings.
///
/// # Errors
///
/// Returns an error if the start directory does not exist or cannot be read.
pub fn enumerate_files(
    directory: &Path,
    pattern: &WildcardPattern,
) -> Result<SearchResult<PathBuf>, anyhow::Error> {
    if !directory.is_dir() {
        anyhow::bail!("directory '{}' does not exist", directory.display());
    }

    let mut result = SearchResult::default();
    // The sort is stable, so files and directories keep their enumeration order.
    let walker =
        WalkDir::new(directory).sort_by(|a, b| a.file_type().is_dir().cmp(&b.file_type().is_dir()));
    for entry in walker {
        match entry {
            Ok(entry) => {
                if entry.file_type().is_file()
                    && pattern.matches(&entry.file_name().to_string_lossy())
                {
                    result.items.push(entry.into_path());
                }
            }
            Err(err) if err.depth() == 0 => {
                anyhow::bail!("cannot read directory '{}': {}", directory.display(), err);
            }
            Err(err) => result.warnings.push(walk_error_message(&err)),
        }
    }
    Ok(result)
}

fn walk_error_message(err: &walkdir::Error) -> String {
    let reason = err
        .io_error()
        .map_or_else(|| err.to_string(), ToString::to_string);
    match err.path() {
        Some(path) => format!("skipped '{}': {}", path.display(), reason),
        None => format!("skipped entry: {reason}"),
    }
}

/// Creates file information, reading the file version if `read_version` is `true`.
///
/// Files whose version cannot be read because they are inaccessible are skipped and
/// reported as warnings.
pub fn collect_file_info(files: Vec<PathBuf>, read_version: bool) -> SearchResult<FileInfo> {
    let mut result = SearchResult::default();
    for path in files {
        let version = if read_version {
            match read_file_version(&path) {
                Ok(version) => version,
                Err(err) => {
                    result
                        .warnings
                        .push(format!("skipped '{}': {}", path.display(), err));
                    continue;
                }
            }
        } else {
            None
        };
        result.items.push(FileInfo { path, version });
    }
    result
}

/// Keeps only files whose version lies within the inclusive bounds.
///
/// Files whose version is missing or not comparable are omitted.
pub fn filter_by_version(
    files: Vec<FileInfo>,
    min_version: Option<FileVersion>,
    max_version: Option<FileVersion>,
) -> Vec<FileInfo> {
    files
        .into_iter()
        .filter(|file| {
            let Some(version) = file.version.as_deref().and_then(FileVersion::parse_lenient) else {
                return false;
            };
            min_version.is_none_or(|min| version >= min)
                && max_version.is_none_or(|max| version <= max)
        })
        .collect()
}

/// Sorts files by full path, case-insensitively; paths differing only in case are ordered ordinally.
pub fn sort_by_path(mut files: Vec<FileInfo>) -> Vec<FileInfo> {
    files.sort_by(|a, b| compare_paths(&a.path, &b.path));
    files
}

/// Sorts files by version in ascending order; equal versions are ordered by path.
///
/// Files without a comparable version come last, ordered by path.
pub fn sort_by_version(files: Vec<FileInfo>) -> Vec<FileInfo> {
    let mut keyed: Vec<_> = files
        .into_iter()
        .map(|file| {
            let version = file.version.as_deref().and_then(FileVersion::parse_lenient);
            (version, file)
        })
        .collect();
    keyed.sort_by(|(version_a, a), (version_b, b)| {
        let by_version = match (version_a, version_b) {
            (Some(x), Some(y)) => x.cmp(y),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        };
        by_version.then_with(|| compare_paths(&a.path, &b.path))
    });
    keyed.into_iter().map(|(_, file)| file).collect()
}

fn compare_paths(a: &Path, b: &Path) -> Ordering {
    let (a, b) = (a.to_string_lossy(), b.to_string_lossy());
    fold_case(&a).cmp(&fold_case(&b)).then_with(|| a.cmp(&b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn file(path: &str, version: Option<&str>) -> FileInfo {
        FileInfo {
            path: PathBuf::from(path),
            version: version.map(str::to_string),
        }
    }

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(name);
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn wildcard_star_and_question_mark() {
        let pattern = WildcardPattern::new("te?t*.dll");
        assert!(pattern.matches("test.dll"));
        assert!(pattern.matches("text-2.dll"));
        assert!(!pattern.matches("tst.dll"));
        assert!(!pattern.matches("test.dll.bak"));
    }

    #[test]
    fn wildcard_is_case_insensitive() {
        assert!(WildcardPattern::new("*.PDB").matches("flist.pdb"));
        assert!(WildcardPattern::new("Ä*").matches("äpfel.txt"));
    }

    #[test]
    fn wildcard_star_matches_everything() {
        let pattern = WildcardPattern::new("*");
        assert!(pattern.matches("a"));
        assert!(pattern.matches("no-extension"));
    }

    #[test]
    fn wildcard_treats_brackets_literally() {
        assert!(WildcardPattern::new("[a].txt").matches("[a].txt"));
        assert!(!WildcardPattern::new("[a].txt").matches("a.txt"));
    }

    #[test]
    fn wildcard_backtracks_over_multiple_stars() {
        assert!(WildcardPattern::new("*a*b*c").matches("xxaxxbxxbxxc"));
        assert!(!WildcardPattern::new("*a*b*c").matches("xxaxxcxxb"));
    }

    #[test]
    fn enumerate_files_lists_files_before_subdirectories() {
        let dir = temp_dir("flist_test_order");
        fs::create_dir_all(dir.join("a").join("a1")).unwrap();
        fs::create_dir_all(dir.join("b")).unwrap();
        for file in ["r.txt", "a/y.txt", "a/a1/x.txt", "b/z.txt"] {
            fs::File::create(dir.join(file)).unwrap();
        }

        let result = enumerate_files(&dir, &WildcardPattern::new("*")).unwrap();
        let names: Vec<String> = result
            .items
            .iter()
            .map(|path| {
                path.strip_prefix(&dir)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect();
        let position = |name: &str| names.iter().position(|n| n == name).unwrap();

        assert_eq!(position("r.txt"), 0);
        assert_eq!(position("a/a1/x.txt"), position("a/y.txt") + 1);
        assert_eq!(names.len(), 4);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn enumerate_files_recursively_with_pattern() {
        let dir = temp_dir("flist_test_enumerate");
        fs::create_dir_all(dir.join("sub")).unwrap();
        fs::File::create(dir.join("root.TXT")).unwrap();
        fs::File::create(dir.join("other.exe")).unwrap();
        fs::File::create(dir.join("sub").join("nested.txt")).unwrap();

        let result = enumerate_files(&dir, &WildcardPattern::new("*.txt")).unwrap();
        assert_eq!(result.items.len(), 2);
        assert!(result.warnings.is_empty());

        let result = enumerate_files(&dir, &WildcardPattern::new("*")).unwrap();
        assert_eq!(result.items.len(), 3);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn enumerate_files_fails_for_missing_directory() {
        let dir = std::env::temp_dir().join("flist_test_does_not_exist");
        assert!(enumerate_files(&dir, &WildcardPattern::new("*")).is_err());
    }

    #[test]
    fn collect_file_info_without_version() {
        let result = collect_file_info(vec![PathBuf::from("does-not-matter.txt")], false);
        assert_eq!(result.items, vec![file("does-not-matter.txt", None)]);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn collect_file_info_warns_about_unreadable_file() {
        let result = collect_file_info(vec![PathBuf::from("flist-missing-file.dll")], true);
        assert!(result.items.is_empty());
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn filter_bounds_are_inclusive() {
        let files = vec![
            file("1.dll", Some("1.0.0.0")),
            file("2.dll", Some("2.0.0.0")),
            file("3.dll", Some("3.0.0.0")),
        ];
        let filtered = filter_by_version(files.clone(), Some("2.0".parse().unwrap()), None);
        assert_eq!(filtered, files[1..].to_vec());

        let filtered = filter_by_version(files.clone(), None, Some("2.0".parse().unwrap()));
        assert_eq!(filtered, files[..2].to_vec());

        let filtered = filter_by_version(
            files.clone(),
            Some("1.5".parse().unwrap()),
            Some("2.5".parse().unwrap()),
        );
        assert_eq!(filtered, vec![files[1].clone()]);
    }

    #[test]
    fn filter_treats_missing_parts_as_zero() {
        let files = vec![file("a.exe", Some("14.08"))];
        let filtered = filter_by_version(
            files.clone(),
            Some("14.8.0.0".parse().unwrap()),
            Some("14.8".parse().unwrap()),
        );
        assert_eq!(filtered, files);
    }

    #[test]
    fn filter_omits_files_without_comparable_version() {
        let files = vec![
            file("none.dll", None),
            file("text.dll", Some("beta")),
            file("lenient.dll", Some("1.0.0-beta")),
        ];
        let filtered = filter_by_version(files, Some("0".parse().unwrap()), None);
        assert_eq!(filtered, vec![file("lenient.dll", Some("1.0.0-beta"))]);
    }

    #[test]
    fn sort_by_version_numerically_then_by_path() {
        let files = vec![
            file("none.dll", None),
            file("b10.dll", Some("1.10")),
            file("text.dll", Some("beta")),
            file("b2.dll", Some("1.2.0.0")),
            file("a2.dll", Some("1.2")),
            file("old.dll", Some("0.9.4.0")),
        ];
        let sorted: Vec<_> = sort_by_version(files)
            .into_iter()
            .map(|f| f.path.to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            sorted,
            [
                "old.dll", "a2.dll", "b2.dll", "b10.dll", "none.dll", "text.dll"
            ]
        );
    }

    #[test]
    fn sort_is_case_insensitive_then_ordinal() {
        let files = vec![
            file("b.txt", None),
            file("a.txt", None),
            file("B.txt", None),
            file("C.txt", None),
        ];
        let sorted: Vec<_> = sort_by_path(files)
            .into_iter()
            .map(|f| f.path.to_string_lossy().into_owned())
            .collect();
        assert_eq!(sorted, ["a.txt", "B.txt", "b.txt", "C.txt"]);
    }
}
