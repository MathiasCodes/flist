//! File version representation and parsing.
//!
//! This module provides the [`FileVersion`] struct for representing and comparing
//! file versions in the format `major.minor.build.private`.

use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

/// Represents a file version with up to 4 parts: major.minor.build.private.
///
/// Each part is optional and represented as `Option<u32>`. This allows for partial
/// versions like "1.0" or "2.3.4".
///
/// # Examples
///
/// ```
/// use std::str::FromStr;
/// use flist::file_version::FileVersion;
///
/// let version = FileVersion::from_str("1.2.3.4").unwrap();
/// assert_eq!(version.major, Some(1));
/// assert_eq!(version.minor, Some(2));
/// assert_eq!(version.build, Some(3));
/// assert_eq!(version.private, Some(4));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileVersion {
    pub major: Option<u32>,
    pub minor: Option<u32>,
    pub build: Option<u32>,
    pub private: Option<u32>,
}

impl FileVersion {
    /// Creates a new `FileVersion` with the specified parts.
    ///
    /// # Arguments
    ///
    /// * `major` - The major version number
    /// * `minor` - The minor version number
    /// * `build` - The build number
    /// * `private` - The private/revision number
    ///
    /// # Examples
    ///
    /// ```
    /// use flist::file_version::FileVersion;
    ///
    /// let version = FileVersion::new(Some(1), Some(2), Some(3), Some(4));
    /// assert_eq!(version.to_string(), "1.2.3.4");
    /// ```
    pub fn new(
        major: Option<u32>,
        minor: Option<u32>,
        build: Option<u32>,
        private: Option<u32>,
    ) -> Self {
        Self {
            major,
            minor,
            build,
            private,
        }
    }
}

impl FromStr for FileVersion {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('.').collect();

        let major = if !parts.is_empty() && !parts[0].is_empty() {
            Some(parts[0].parse::<u32>()?)
        } else {
            None
        };

        let minor = if parts.len() > 1 && !parts[1].is_empty() {
            Some(parts[1].parse::<u32>()?)
        } else {
            None
        };

        let build = if parts.len() > 2 && !parts[2].is_empty() {
            Some(parts[2].parse::<u32>()?)
        } else {
            None
        };

        let private = if parts.len() > 3 && !parts[3].is_empty() {
            Some(parts[3].parse::<u32>()?)
        } else {
            None
        };

        Ok(FileVersion {
            major,
            minor,
            build,
            private,
        })
    }
}

impl PartialOrd for FileVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FileVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare major
        match self.major.cmp(&other.major) {
            Ordering::Equal => {}
            ord => return ord,
        }

        // Compare minor
        match self.minor.cmp(&other.minor) {
            Ordering::Equal => {}
            ord => return ord,
        }

        // Compare build
        match self.build.cmp(&other.build) {
            Ordering::Equal => {}
            ord => return ord,
        }

        // Compare private
        self.private.cmp(&other.private)
    }
}

impl fmt::Display for FileVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let major = self.major.map_or(0, |v| v);
        let minor = self.minor.map_or(0, |v| v);
        let build = self.build.map_or(0, |v| v);
        let private = self.private.map_or(0, |v| v);

        write!(f, "{}.{}.{}.{}", major, minor, build, private)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_version() {
        let version = FileVersion::from_str("1.2.3.4").unwrap();
        assert_eq!(version.major, Some(1));
        assert_eq!(version.minor, Some(2));
        assert_eq!(version.build, Some(3));
        assert_eq!(version.private, Some(4));
    }

    #[test]
    fn test_parse_partial_version() {
        let version = FileVersion::from_str("1.2").unwrap();
        assert_eq!(version.major, Some(1));
        assert_eq!(version.minor, Some(2));
        assert_eq!(version.build, None);
        assert_eq!(version.private, None);
    }

    #[test]
    fn test_version_comparison() {
        let v1 = FileVersion::from_str("1.0.0.0").unwrap();
        let v2 = FileVersion::from_str("1.0.0.1").unwrap();
        let v3 = FileVersion::from_str("1.0.1.0").unwrap();
        let v4 = FileVersion::from_str("1.1.0.0").unwrap();
        let v5 = FileVersion::from_str("2.0.0.0").unwrap();

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v3 < v4);
        assert!(v4 < v5);
    }

    #[test]
    fn test_version_equality() {
        let v1 = FileVersion::from_str("1.2.3.4").unwrap();
        let v2 = FileVersion::from_str("1.2.3.4").unwrap();
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_partial_version_equality() {
        let v1 = FileVersion::from_str("1.2").unwrap();
        let v2 = FileVersion::new(Some(1), Some(2), None, None);
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_display_format() {
        let version = FileVersion::from_str("1.2.3.4").unwrap();
        assert_eq!(format!("{}", version), "1.2.3.4");

        let partial = FileVersion::from_str("1.2").unwrap();
        assert_eq!(format!("{}", partial), "1.2.0.0");
    }

    #[test]
    fn test_invalid_version() {
        let result = FileVersion::from_str("abc");
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_string() {
        let result = FileVersion::from_str("");
        assert!(result.is_ok());
        let version = result.unwrap();
        assert_eq!(version.major, None);
    }

    #[test]
    fn test_greater_than_operator() {
        let v1 = FileVersion::from_str("2.0.0.0").unwrap();
        let v2 = FileVersion::from_str("1.0.0.0").unwrap();
        assert!(v1 > v2);
    }

    #[test]
    fn test_less_than_or_equal() {
        let v1 = FileVersion::from_str("1.0.0.0").unwrap();
        let v2 = FileVersion::from_str("1.0.0.0").unwrap();
        let v3 = FileVersion::from_str("2.0.0.0").unwrap();
        assert!(v1 <= v2);
        assert!(v1 <= v3);
    }
}
