//! Version extraction from PE (Portable Executable) files.
//!
//! The PE format is parsed directly, so Windows executables and libraries can be read on
//! every operating system. Other files have no version.

use clap::ValueEnum;
use pelite::FileMap;
use pelite::image::VS_VERSION;
use pelite::resources::Resources;
use pelite::resources::version_info::VersionInfo;
use std::io;
use std::path::Path;

/// The value of the version resource that is used as the version of a file.
#[derive(ValueEnum, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum VersionSource {
    /// `FileVersion` string, as shown by Windows Explorer
    #[default]
    File,
    /// `ProductVersion` string
    Product,
    /// Numeric file version from the fixed part of the version resource
    FileFixed,
    /// Numeric product version from the fixed part of the version resource
    ProductFixed,
}

/// Reads the version of a file from the given source.
///
/// # Returns
///
/// * `Ok(Some(version))` - The file has a non-empty value for `source`
/// * `Ok(None)` - The file is not a PE file or has no value for `source`
/// * `Err(_)` - The file could not be opened or read
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use flist::version_reader::{read_version, VersionSource};
///
/// let path = Path::new("C:\\Windows\\System32\\kernel32.dll");
/// if let Some(version) = read_version(path, VersionSource::File).unwrap() {
///     println!("Version: {version}");
/// }
/// ```
pub fn read_version(path: &Path, source: VersionSource) -> io::Result<Option<String>> {
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
        .and_then(|resources| version_from_resources(&resources, source)))
}

fn version_from_resources(resources: &Resources<'_>, source: VersionSource) -> Option<String> {
    let version_info = resources.version_info().ok()?;
    match source {
        VersionSource::File => version_string(&version_info, "FileVersion"),
        VersionSource::Product => version_string(&version_info, "ProductVersion"),
        VersionSource::FileFixed => version_info
            .fixed()
            .map(|fixed| format_fixed(fixed.dwFileVersion)),
        VersionSource::ProductFixed => version_info
            .fixed()
            .map(|fixed| format_fixed(fixed.dwProductVersion)),
    }
}

/// Looks up a version string, preferring the languages listed in the translation table.
fn version_string(version_info: &VersionInfo<'_>, key: &str) -> Option<String> {
    let from_translation = version_info
        .translation()
        .iter()
        .find_map(|&lang| version_info.value(lang, key));
    let value = from_translation.or_else(|| {
        let mut tables: Vec<_> = version_info.file_info().strings.into_iter().collect();
        tables.sort_by_key(|(lang, _)| (lang.lang_id, lang.charset_id));
        tables
            .into_iter()
            .find_map(|(_, mut strings)| strings.remove(key))
    })?;

    let value = value.trim_end_matches('\0').trim().to_string();
    (!value.is_empty()).then_some(value)
}

/// Formats a fixed version as `major.minor.build.private`.
fn format_fixed(version: VS_VERSION) -> String {
    // In memory the version consists of the 16-bit words minor, major, private and build.
    let [minor, major, private, build] =
        unsafe { std::mem::transmute::<VS_VERSION, [u16; 4]>(version) };
    format!("{major}.{minor}.{build}.{private}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    const ALL_SOURCES: [VersionSource; 4] = [
        VersionSource::File,
        VersionSource::Product,
        VersionSource::FileFixed,
        VersionSource::ProductFixed,
    ];

    #[test]
    fn nonexistent_file_is_error() {
        let result = read_version(&PathBuf::from("nonexistent.dll"), VersionSource::File);
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

        for source in ALL_SOURCES {
            assert_eq!(read_version(&text, source).unwrap(), None);
            assert_eq!(read_version(&empty, source).unwrap(), None);
        }

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    #[cfg(windows)]
    fn system_dll_has_version_in_every_source() {
        let system_root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
        let dll_path = PathBuf::from(system_root)
            .join("System32")
            .join("kernel32.dll");

        for source in ALL_SOURCES {
            let version = read_version(&dll_path, source).unwrap();
            assert!(
                version.as_deref().is_some_and(|v| v.starts_with("10.0.")),
                "{source:?}: {version:?}"
            );
        }

        let fixed = read_version(&dll_path, VersionSource::FileFixed)
            .unwrap()
            .unwrap();
        assert_eq!(fixed.split('.').count(), 4);
    }
}
