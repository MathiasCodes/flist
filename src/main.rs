use clap::Parser;
use flist::cli::CliArgs;
use flist::file_lister;
use flist::file_version::FileVersion;
use flist::output;
use std::path::PathBuf;
use std::str::FromStr;

/// Main entry point for the FList application.
///
/// Parses command-line arguments, enumerates files matching the pattern,
/// optionally extracts version information, filters and sorts results,
/// and outputs to console and/or file.
fn main() -> Result<(), anyhow::Error> {
    let mut args = CliArgs::parse();

    // Auto-enable version info if min/max version specified
    args.normalize();

    // Parse version strings
    let min_version = args
        .min_version
        .as_ref()
        .map(|s| {
            FileVersion::from_str(s).map_err(|e| {
                anyhow::anyhow!(
                    "Invalid minimum version '{}': {}. Expected format: major.minor.build.private (e.g., 1.2.3.4)",
                    s, e
                )
            })
        })
        .transpose()?;

    let max_version = args
        .max_version
        .as_ref()
        .map(|s| {
            FileVersion::from_str(s).map_err(|e| {
                anyhow::anyhow!(
                    "Invalid maximum version '{}': {}. Expected format: major.minor.build.private (e.g., 1.2.3.4)",
                    s, e
                )
            })
        })
        .transpose()?;

    // Determine working directory
    let directory = args
        .directory
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    // Print header
    if !args.quiet {
        println!(
            "List files in \"{}\" and its subdirectories.",
            directory.display()
        );
        println!("Use \"flist --help\" to print help.");
        println!();
    }

    // Enumerate files
    let files = file_lister::enumerate_files(&directory, &args.pattern)
        .map_err(|e| anyhow::anyhow!("Failed to enumerate files: {}", e))?;

    // Collect file info with versions
    let mut file_infos = file_lister::collect_file_info(files, args.include_file_version);

    // Filter by version
    if min_version.is_some() || max_version.is_some() {
        file_infos = file_lister::filter_by_version(file_infos, min_version, max_version);
    }

    // Sort if requested
    if args.sort_by_path {
        file_infos = file_lister::sort_by_path(file_infos);
    }

    // Output to console
    output::print_results(&file_infos, args.include_file_version, args.quiet);

    // Output to file if specified
    if let Some(output_file) = args.output_file {
        output::write_to_file(
            &file_infos,
            &PathBuf::from(&output_file),
            args.include_file_version,
        )
        .map_err(|e| anyhow::anyhow!("Failed to write to output file '{}': {}", output_file, e))?;
    }

    Ok(())
}
