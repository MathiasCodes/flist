//! File version extraction from PE (Portable Executable) files.
//!
//! The file version is the `FileVersion` string from the `StringFileInfo` block of the
//! version resource, exactly as stored in the file (e.g. `14.08` or `0.1.0`).

use pelite::FileMap;
use pelite::resources::Resources;
use std::io;
use std::path::Path;

/// Name of the version resource string that holds the file version.
const FILE_VERSION_KEY: &str = "FileVersion";

/// Reads the `FileVersion` string of a file.
///
/// # Returns
///
/// * `Ok(Some(version))` - The file has a non-empty `FileVersion` string
/// * `Ok(None)` - The file is not a PE file or has no `FileVersion` string
/// * `Err(_)` - The file could not be opened or read
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use flist::version_reader::read_file_version;
///
/// if let Some(version) = read_file_version(Path::new("C:\\Windows\\System32\\kernel32.dll")).unwrap() {
///     println!("Version: {version}");
/// }
/// ```
pub fn read_file_version(path: &Path) -> io::Result<Option<String>> {
    if std::fs::metadata(path)?.len() == 0 {
        return Ok(None);
    }
    let file_map = FileMap::open(path)?;
    let bytes = file_map.as_ref();

    let resources = if let Ok(pe) = pelite::pe64::PeFile::from_bytes(bytes) {
        pelite::pe64::Pe::resources(pe)
    } else if let Ok(pe) = pelite::pe32::PeFile::from_bytes(bytes) {
        pelite::pe32::Pe::resources(pe)
    } else {
        return Ok(None);
    };

    Ok(resources
        .ok()
        .and_then(|resources| file_version_string(&resources)))
}

/// Looks up the `FileVersion` string, preferring the languages listed in the translation table.
fn file_version_string(resources: &Resources<'_>) -> Option<String> {
    let version_info = resources.version_info().ok()?;
    let from_translation = version_info
        .translation()
        .iter()
        .find_map(|&lang| version_info.value(lang, FILE_VERSION_KEY));
    let value = from_translation.or_else(|| {
        let mut tables: Vec<_> = version_info.file_info().strings.into_iter().collect();
        tables.sort_by_key(|(lang, _)| (lang.lang_id, lang.charset_id));
        tables
            .into_iter()
            .find_map(|(_, mut strings)| strings.remove(FILE_VERSION_KEY))
    })?;

    let value = value.trim_end_matches('\0').trim().to_string();
    (!value.is_empty()).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn nonexistent_file_is_error() {
        let result = read_file_version(&PathBuf::from("nonexistent.dll"));
        assert!(result.is_err());
    }

    #[test]
    fn non_pe_file_has_no_version() {
        let dir = std::env::temp_dir().join("flist_test_version_reader");
        fs::create_dir_all(&dir).unwrap();
        let text = dir.join("text.txt");
        let empty = dir.join("empty.dll");
        fs::write(&text, "not a PE file").unwrap();
        fs::write(&empty, "").unwrap();

        assert_eq!(read_file_version(&text).unwrap(), None);
        assert_eq!(read_file_version(&empty).unwrap(), None);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    #[cfg(windows)]
    fn system_dll_has_version() {
        let system_root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
        let dll_path = PathBuf::from(system_root)
            .join("System32")
            .join("kernel32.dll");

        let version = read_file_version(&dll_path).unwrap();
        assert!(version.is_some_and(|v| v.starts_with("10.")));
    }
}
