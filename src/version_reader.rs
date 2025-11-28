//! File version extraction from PE (Portable Executable) files.
//!
//! This module provides functionality to extract version information from Windows
//! executable files (.exe) and dynamic link libraries (.dll) by parsing the PE
//! file format and reading the VS_FIXEDFILEINFO structure.

use crate::file_version::FileVersion;
use std::path::Path;

/// Reads file version information from a PE file (Windows executable or DLL).
///
/// This function attempts to parse the file as a PE (Portable Executable) file
/// and extract version information from the VS_FIXEDFILEINFO structure in the
/// file's resources.
///
/// # Arguments
///
/// * `path` - Path to the file to read
///
/// # Returns
///
/// * `Ok(Some(FileVersion))` - Version information was successfully extracted
/// * `Ok(None)` - File is not a PE file or has no version information
/// * `Err(_)` - An error occurred while reading the file
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use flist::version_reader::read_file_version;
///
/// let version = read_file_version(Path::new("C:\\Windows\\System32\\kernel32.dll")).unwrap();
/// if let Some(v) = version {
///     println!("Version: {}", v);
/// }
/// ```
#[cfg(windows)]
pub fn read_file_version(path: &Path) -> Result<Option<FileVersion>, anyhow::Error> {
    use pelite::pe32::PeFile as PeFile32;
    use pelite::pe64::PeFile;
    use pelite::FileMap;

    // Try to read the file
    let file_map = match FileMap::open(path) {
        Ok(map) => map,
        Err(_) => return Ok(None), // Not a valid file or can't read
    };

    // Try as 64-bit PE first
    if let Ok(pe) = PeFile::from_bytes(&file_map) {
        return extract_version_from_pe64(pe);
    }

    // Try as 32-bit PE
    if let Ok(pe) = PeFile32::from_bytes(&file_map) {
        return extract_version_from_pe32(pe);
    }

    // Not a PE file
    Ok(None)
}

#[cfg(windows)]
fn extract_version_from_pe64(
    pe: pelite::pe64::PeFile,
) -> Result<Option<FileVersion>, anyhow::Error> {
    use pelite::pe64::Pe;
    use pelite::resources::FindError;

    // Get resources
    let resources = match pe.resources() {
        Ok(res) => res,
        Err(_) => return Ok(None), // No resources
    };

    // Try to find version info
    let version_info = match resources.version_info() {
        Ok(vi) => vi,
        Err(FindError::NotFound) => return Ok(None), // No version info
        Err(_) => return Ok(None),                   // Other error
    };

    // Get the fixed file info which contains the version
    let fixed = match version_info.fixed() {
        Some(f) => f,
        None => return Ok(None),
    };

    // Extract version parts from dwFileVersion
    // VS_VERSION is a u64 where (in little-endian):
    // - Bits 0-15: minor (low word of MS dword)
    // - Bits 16-31: major (high word of MS dword)
    // - Bits 32-47: private (low word of LS dword)
    // - Bits 48-63: build (high word of LS dword)
    let file_ver_raw =
        unsafe { std::mem::transmute::<pelite::image::VS_VERSION, u64>(fixed.dwFileVersion) };
    let minor = (file_ver_raw & 0xFFFF) as u32;
    let major = ((file_ver_raw >> 16) & 0xFFFF) as u32;
    let private = ((file_ver_raw >> 32) & 0xFFFF) as u32;
    let build = ((file_ver_raw >> 48) & 0xFFFF) as u32;

    Ok(Some(FileVersion::new(
        Some(major),
        Some(minor),
        Some(build),
        Some(private),
    )))
}

#[cfg(windows)]
fn extract_version_from_pe32(
    pe: pelite::pe32::PeFile,
) -> Result<Option<FileVersion>, anyhow::Error> {
    use pelite::pe32::Pe;
    use pelite::resources::FindError;

    // Get resources
    let resources = match pe.resources() {
        Ok(res) => res,
        Err(_) => return Ok(None), // No resources
    };

    // Try to find version info
    let version_info = match resources.version_info() {
        Ok(vi) => vi,
        Err(FindError::NotFound) => return Ok(None), // No version info
        Err(_) => return Ok(None),                   // Other error
    };

    // Get the fixed file info which contains the version
    let fixed = match version_info.fixed() {
        Some(f) => f,
        None => return Ok(None),
    };

    // Extract version parts from dwFileVersion
    let file_ver_raw =
        unsafe { std::mem::transmute::<pelite::image::VS_VERSION, u64>(fixed.dwFileVersion) };
    let minor = (file_ver_raw & 0xFFFF) as u32;
    let major = ((file_ver_raw >> 16) & 0xFFFF) as u32;
    let private = ((file_ver_raw >> 32) & 0xFFFF) as u32;
    let build = ((file_ver_raw >> 48) & 0xFFFF) as u32;

    Ok(Some(FileVersion::new(
        Some(major),
        Some(minor),
        Some(build),
        Some(private),
    )))
}

/// Read file version information from a PE file (cross-platform stub)
/// On non-Windows platforms, this can still read PE files using pelite
#[cfg(not(windows))]
pub fn read_file_version(path: &Path) -> Result<Option<FileVersion>, anyhow::Error> {
    use pelite::pe32::PeFile as PeFile32;
    use pelite::pe64::PeFile;
    use pelite::FileMap;

    // Try to read the file
    let file_map = match FileMap::open(path) {
        Ok(map) => map,
        Err(_) => return Ok(None), // Not a valid file or can't read
    };

    // Try as 64-bit PE first
    if let Ok(pe) = PeFile::from_bytes(&file_map) {
        return extract_version_from_pe64_cross(pe);
    }

    // Try as 32-bit PE
    if let Ok(pe) = PeFile32::from_bytes(&file_map) {
        return extract_version_from_pe32_cross(pe);
    }

    // Not a PE file
    Ok(None)
}

#[cfg(not(windows))]
fn extract_version_from_pe64_cross(
    pe: pelite::pe64::PeFile,
) -> Result<Option<FileVersion>, anyhow::Error> {
    use pelite::pe64::Pe;
    use pelite::resources::FindError;

    let resources = match pe.resources() {
        Ok(res) => res,
        Err(_) => return Ok(None),
    };

    let version_info = match resources.version_info() {
        Ok(vi) => vi,
        Err(FindError::NotFound) => return Ok(None),
        Err(_) => return Ok(None),
    };

    let fixed = match version_info.fixed() {
        Some(f) => f,
        None => return Ok(None),
    };

    let file_ver_raw =
        unsafe { std::mem::transmute::<pelite::image::VS_VERSION, u64>(fixed.dwFileVersion) };
    let minor = (file_ver_raw & 0xFFFF) as u32;
    let major = ((file_ver_raw >> 16) & 0xFFFF) as u32;
    let private = ((file_ver_raw >> 32) & 0xFFFF) as u32;
    let build = ((file_ver_raw >> 48) & 0xFFFF) as u32;

    Ok(Some(FileVersion::new(
        Some(major),
        Some(minor),
        Some(build),
        Some(private),
    )))
}

#[cfg(not(windows))]
fn extract_version_from_pe32_cross(
    pe: pelite::pe32::PeFile,
) -> Result<Option<FileVersion>, anyhow::Error> {
    use pelite::pe32::Pe;
    use pelite::resources::FindError;

    let resources = match pe.resources() {
        Ok(res) => res,
        Err(_) => return Ok(None),
    };

    let version_info = match resources.version_info() {
        Ok(vi) => vi,
        Err(FindError::NotFound) => return Ok(None),
        Err(_) => return Ok(None),
    };

    let fixed = match version_info.fixed() {
        Some(f) => f,
        None => return Ok(None),
    };

    let file_ver_raw =
        unsafe { std::mem::transmute::<pelite::image::VS_VERSION, u64>(fixed.dwFileVersion) };
    let minor = (file_ver_raw & 0xFFFF) as u32;
    let major = ((file_ver_raw >> 16) & 0xFFFF) as u32;
    let private = ((file_ver_raw >> 32) & 0xFFFF) as u32;
    let build = ((file_ver_raw >> 48) & 0xFFFF) as u32;

    Ok(Some(FileVersion::new(
        Some(major),
        Some(minor),
        Some(build),
        Some(private),
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    #[cfg(windows)]
    fn test_read_version_from_system_dll() {
        // Test with a known Windows system DLL
        let system_root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
        let dll_path = PathBuf::from(system_root)
            .join("System32")
            .join("kernel32.dll");

        if dll_path.exists() {
            let result = read_file_version(&dll_path);
            assert!(result.is_ok());

            let version = result.unwrap();
            // kernel32.dll should have version info
            assert!(version.is_some());

            if let Some(v) = version {
                // Version should have at least major and minor
                assert!(v.major.is_some());
                assert!(v.minor.is_some());
                println!("kernel32.dll version: {}", v);
            }
        }
    }

    #[test]
    fn test_read_version_from_nonexistent_file() {
        let path = PathBuf::from("nonexistent.dll");
        let result = read_file_version(&path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }
}
