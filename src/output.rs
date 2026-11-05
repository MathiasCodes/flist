//! Output formatting for console and file output.

use crate::file_lister::FileInfo;
use std::io::{self, Write};

/// Width of the version column, including the separating spaces.
const VERSION_COLUMN_WIDTH: usize = 16;

/// Line terminator of every output line.
const LINE_ENDING: &str = "\r\n";

/// Formats one output line without the line terminator.
///
/// With `include_version` the version is left-aligned in a column of 16 characters,
/// followed by the full path; a version of 16 or more characters is followed by a single space.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use flist::file_lister::FileInfo;
/// use flist::output::format_line;
///
/// let file = FileInfo { path: PathBuf::from("C:\\a.dll"), version: Some("1.0".to_string()) };
/// assert_eq!(format_line(&file, true), "1.0             C:\\a.dll");
/// assert_eq!(format_line(&file, false), "C:\\a.dll");
/// ```
pub fn format_line(file: &FileInfo, include_version: bool) -> String {
    let path = file.path.display();
    if !include_version {
        return path.to_string();
    }
    let version = file.version.as_deref().unwrap_or("");
    if version.chars().count() >= VERSION_COLUMN_WIDTH {
        format!("{version} {path}")
    } else {
        format!("{version:<VERSION_COLUMN_WIDTH$}{path}")
    }
}

/// Writes all files as CRLF-terminated lines to `writer`.
pub fn write_lines(
    writer: &mut impl Write,
    files: &[FileInfo],
    include_version: bool,
) -> io::Result<()> {
    for file in files {
        write!(
            writer,
            "{}{LINE_ENDING}",
            format_line(file, include_version)
        )?;
    }
    writer.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn file(path: &str, version: Option<&str>) -> FileInfo {
        FileInfo {
            path: PathBuf::from(path),
            version: version.map(str::to_string),
        }
    }

    #[test]
    fn version_column_is_padded_to_sixteen_characters() {
        assert_eq!(
            format_line(&file("C:\\a.exe", Some("14.08")), true),
            "14.08           C:\\a.exe"
        );
    }

    #[test]
    fn missing_version_leaves_column_blank() {
        assert_eq!(
            format_line(&file("C:\\a.exe", None), true),
            format!("{}C:\\a.exe", " ".repeat(16))
        );
    }

    #[test]
    fn long_version_is_followed_by_single_space() {
        assert_eq!(
            format_line(&file("C:\\a.exe", Some("10.0.26100.7019x")), true),
            "10.0.26100.7019x C:\\a.exe"
        );
    }

    #[test]
    fn path_only_without_version() {
        assert_eq!(
            format_line(&file("C:\\a.exe", Some("1.0")), false),
            "C:\\a.exe"
        );
    }

    #[test]
    fn write_lines_uses_crlf() {
        let mut buffer = Vec::new();
        write_lines(&mut buffer, &[file("a", None), file("b", None)], false).unwrap();
        assert_eq!(String::from_utf8(buffer).unwrap(), "a\r\nb\r\n");
    }
}
