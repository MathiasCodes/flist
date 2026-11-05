use clap::Parser;
use flist::cli::{CliArgs, SortKey};
use flist::file_lister::{self, WildcardPattern};
use flist::output;
use std::fs::File;
use std::io::{self, BufWriter};
use std::process::ExitCode;

/// Exit code for runtime errors and skipped inaccessible entries.
const EXIT_RUNTIME_ERROR: u8 = 1;

/// Main entry point for the FList application.
///
/// Usage errors are reported by clap with exit code 2.
fn main() -> ExitCode {
    let args = CliArgs::parse();
    match run(&args) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(EXIT_RUNTIME_ERROR),
        Err(err) => {
            eprintln!("flist: error: {err:#}");
            ExitCode::from(EXIT_RUNTIME_ERROR)
        }
    }
}

/// Lists the files; returns `Ok(false)` if inaccessible entries were skipped with a warning.
fn run(args: &CliArgs) -> Result<bool, anyhow::Error> {
    let directory = match &args.dir {
        Some(dir) => std::path::absolute(dir)?,
        None => std::env::current_dir()?,
    };
    let pattern = WildcardPattern::new(&args.pattern);

    let found = file_lister::enumerate_files(&directory, &pattern)?;
    let mut warnings = found.warnings;

    let sort_key = args.sort_key();
    let read_version =
        args.file_version || args.has_version_filter() || sort_key == Some(SortKey::Version);
    let collected = file_lister::collect_file_info(found.items, read_version);
    warnings.extend(collected.warnings);
    let mut files = collected.items;

    if args.has_version_filter() {
        files = file_lister::filter_by_version(files, args.min_version, args.max_version);
    }
    match sort_key {
        Some(SortKey::Path) => files = file_lister::sort_by_path(files),
        Some(SortKey::Version) => files = file_lister::sort_by_version(files),
        None => {}
    }
    if args.reverse {
        files.reverse();
    }

    let report_warnings = !args.ignore_inaccessible && !warnings.is_empty();
    if report_warnings {
        for warning in &warnings {
            eprintln!("flist: warning: {warning}");
        }
    }

    output::write_lines(&mut io::stdout().lock(), &files, args.file_version)?;

    if let Some(output_path) = &args.output {
        File::create(output_path)
            .map(BufWriter::new)
            .and_then(|mut writer| output::write_lines(&mut writer, &files, args.file_version))
            .map_err(|err| {
                anyhow::anyhow!(
                    "cannot write output file '{}': {}",
                    output_path.display(),
                    err
                )
            })?;
    }

    Ok(!report_warnings)
}
