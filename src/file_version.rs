//! File version representation, parsing and comparison.
//!
//! A version has up to four numeric parts: `major.minor.build.private`.
//! Missing parts count as `0`, so `14.08` equals `14.8.0.0`.

use std::fmt;
use std::str::FromStr;

/// A comparable file version with four numeric parts.
///
/// # Examples
///
/// ```
/// use flist::file_version::FileVersion;
///
/// let version: FileVersion = "1.2".parse().unwrap();
/// assert_eq!(version, "1.2.0.0".parse().unwrap());
/// assert_eq!(version.to_string(), "1.2.0.0");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileVersion([u32; 4]);

impl FileVersion {
    /// Creates a version from its four parts.
    pub fn new(major: u32, minor: u32, build: u32, private: u32) -> Self {
        Self([major, minor, build, private])
    }

    /// Interprets a version string read from a file leniently.
    ///
    /// For each part only the leading digits are used; the rest of the part is ignored.
    /// Parts beyond the fourth are ignored.
    /// Returns `None` if the first part has no leading digit.
    /// If a later part has no leading digit, that part and all following parts count as `0`.
    ///
    /// # Examples
    ///
    /// ```
    /// use flist::file_version::FileVersion;
    ///
    /// assert_eq!(FileVersion::parse_lenient("1.0.0-beta"), Some(FileVersion::new(1, 0, 0, 0)));
    /// assert_eq!(FileVersion::parse_lenient("6.0.1 (build 42)"), Some(FileVersion::new(6, 0, 1, 0)));
    /// assert_eq!(FileVersion::parse_lenient("beta"), None);
    /// ```
    pub fn parse_lenient(s: &str) -> Option<Self> {
        let mut parts = [0u32; 4];
        for (index, part) in s.trim().split('.').take(4).enumerate() {
            let digits: String = part.chars().take_while(char::is_ascii_digit).collect();
            if digits.is_empty() {
                if index == 0 {
                    return None;
                }
                break;
            }
            parts[index] = digits.parse().unwrap_or(u32::MAX);
        }
        Some(Self(parts))
    }
}

/// Parses a version argument strictly: one to four purely numeric parts separated by dots.
impl FromStr for FileVersion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let invalid = || {
            format!(
                "invalid version '{s}', expected one to four numbers separated by dots (e.g. 1.2.3.4)"
            )
        };
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() > 4 {
            return Err(invalid());
        }
        let mut result = [0u32; 4];
        for (index, part) in parts.iter().enumerate() {
            if part.is_empty() || !part.chars().all(|c| c.is_ascii_digit()) {
                return Err(invalid());
            }
            result[index] = part.parse().map_err(|_| invalid())?;
        }
        Ok(Self(result))
    }
}

impl fmt::Display for FileVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [major, minor, build, private] = self.0;
        write!(f, "{major}.{minor}.{build}.{private}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_version() {
        assert_eq!("1.2.3.4".parse(), Ok(FileVersion::new(1, 2, 3, 4)));
    }

    #[test]
    fn parse_partial_version_fills_with_zero() {
        assert_eq!("1.2".parse(), Ok(FileVersion::new(1, 2, 0, 0)));
        assert_eq!("14.08".parse(), Ok(FileVersion::new(14, 8, 0, 0)));
    }

    #[test]
    fn parse_rejects_invalid_arguments() {
        for value in [
            "",
            "abc",
            "1.",
            ".1",
            "1..2",
            "1.0-beta",
            "1.2.3.4.5",
            " 1",
            "-1",
        ] {
            assert!(value.parse::<FileVersion>().is_err(), "value {value:?}");
        }
    }

    #[test]
    fn partial_version_equals_full_version() {
        let partial: FileVersion = "1.2".parse().unwrap();
        let full: FileVersion = "1.2.0.0".parse().unwrap();
        assert_eq!(partial, full);
    }

    #[test]
    fn compare_part_by_part_numerically() {
        let versions: Vec<FileVersion> = [
            "1.0.0.0", "1.0.0.1", "1.0.1.0", "1.1.0.0", "2.0.0.0", "10.0.0.0",
        ]
        .iter()
        .map(|v| v.parse().unwrap())
        .collect();
        assert!(versions.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn lenient_uses_leading_digits() {
        assert_eq!(
            FileVersion::parse_lenient("1.1.30.01"),
            Some(FileVersion::new(1, 1, 30, 1))
        );
        assert_eq!(
            FileVersion::parse_lenient("10.0.26100.1 (WinBuild.160101.0800)"),
            Some(FileVersion::new(10, 0, 26100, 1))
        );
        assert_eq!(
            FileVersion::parse_lenient("1.0.0-beta"),
            Some(FileVersion::new(1, 0, 0, 0))
        );
    }

    #[test]
    fn lenient_treats_later_non_numeric_parts_as_zero() {
        assert_eq!(
            FileVersion::parse_lenient("2.x.5.6"),
            Some(FileVersion::new(2, 0, 0, 0))
        );
    }

    #[test]
    fn lenient_ignores_parts_beyond_fourth() {
        assert_eq!(
            FileVersion::parse_lenient("1.2.3.4.5"),
            Some(FileVersion::new(1, 2, 3, 4))
        );
    }

    #[test]
    fn lenient_returns_none_without_leading_digit() {
        assert_eq!(FileVersion::parse_lenient(""), None);
        assert_eq!(FileVersion::parse_lenient("v1.0"), None);
    }

    #[test]
    fn display_shows_four_parts() {
        assert_eq!(FileVersion::new(1, 2, 0, 0).to_string(), "1.2.0.0");
    }
}
