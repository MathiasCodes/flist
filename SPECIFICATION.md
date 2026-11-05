# FList – Specification

This specification describes the behaviour of `flist`.

## Purpose

FList lists files in a directory and all its subdirectories.
Optionally it shows the file version of each file and filters files by version.
A typical use is finding out which versions of DLLs or executables are present in a directory tree.

## Command line

```text
flist [OPTIONS] [PATTERN]
```

| Argument | Meaning |
|---|---|
| `PATTERN` | Search pattern for file names, e.g. `*.dll`. Optional; without it all files are listed. |
| `-f`, `--file-version` | Include the file version in the output. |
| `--sort <KEY>` | Sort the output by `KEY`: `path` or `version`. See [Sorting](#sorting). |
| `-s` | Short form of `--sort path`. |
| `-v` | Short form of `--sort version`. |
| `-r`, `--reverse` | Reverse the sort order. Requires a sort option. |
| `--min-version <VERSION>` | Omit files whose file version is lower than `<VERSION>`. |
| `--max-version <VERSION>` | Omit files whose file version is higher than `<VERSION>`. |
| `-d`, `--dir <DIR>` | Start the search in `<DIR>`. Default: the current directory. |
| `-o`, `--output <FILE>` | Additionally write the output to `<FILE>`. The file is created or overwritten. |
| `--ignore-inaccessible` | Skip inaccessible directories and files silently. |
| `-h`, `--help` | Print the help text. |
| `-V`, `--version` | Print the program version. |

- Options follow GNU conventions: long options start with `--`, short options with `-`.
- Every option that takes a value requires it; the value may be given as `--dir <DIR>` or `--dir=<DIR>`.
  Flags without a value (e.g. `-s`, `-v`) never take one, so they cannot be confused with `PATTERN`.
- Short flags may be combined (`-fsr`).
- At most one `PATTERN` is accepted.
- An unknown option, a missing option value or more than one `PATTERN` is a usage error.

## File search

- The search is recursive: the start directory and all its subdirectories.
- The pattern is matched against the file name only, not against the path.
- The pattern uses Windows wildcard semantics: `*` matches any number of characters, `?` matches exactly one character.
- Matching is case-insensitive (`*.PDB` matches `flist.pdb`).
- Only files are listed, not directories.
- Without a sort option the search is depth-first: within each directory, its files come first, then its subdirectories one after another, each with its complete subtree.
  Apart from that, files and subdirectories appear in file system enumeration order (on NTFS roughly by name, case-insensitively).

  Example with the files `r.txt`, `A\y.txt`, `A\A1\x.txt` and `B\z.txt`:

  ```text
  r.txt
  A\y.txt
  A\A1\x.txt
  B\z.txt
  ```
- With a sort option the files are sorted as described in [Sorting](#sorting).

## Sorting

- `--sort path` (or `-s`) sorts by full path, case-insensitively; paths that differ only in case are ordered ordinally.
- `--sort version` (or `-v`) sorts by file version in ascending order.
  - Versions are interpreted as described in [Version comparison](#version-comparison).
  - Files with equal versions are ordered by full path, as with `--sort path`.
  - Files without a comparable version come last, ordered by full path.
  - The version column is not shown automatically; use `--file-version` for that.
- If several sort options are given, the last one wins (`-s -v` sorts by version).
- `--reverse` reverses the complete sort order, including the position of files without a comparable version.
- `--reverse` without a sort option is a usage error.

### Inaccessible directories and files

- By default, a directory or file that cannot be read (e.g. missing permissions) is skipped, a warning is written to stderr and the search continues.
  At the end the program exits with exit code 1.
- With `--ignore-inaccessible` such entries are skipped without a warning and do not affect the exit code.

## File version

- The file version is the `FileVersion` string from the Windows version resource of the file.
- It is shown exactly as stored in the resource, e.g. `1.0.0.0`, `14.08` or `0.1.0`.
- Files without a version resource have no file version.

### Version comparison

- A version consists of up to four parts separated by dots (major, minor, build, private).
- For each part, the leading digits are used as its numeric value; the rest of the part is ignored.
  Examples: `1.0.0-beta` → `1.0.0`; `6.0.1 (build 42)` → `6.0.1`.
- Missing parts count as `0` (`14.08` equals `14.8.0.0`).
- Parts beyond the fourth are ignored.
- A file version is not comparable if the file has no version or its first part has no leading digit.
  If a later part has no leading digit, that part and all following parts count as `0`.
- Versions are compared part by part numerically.

The arguments of `--min-version` and `--max-version` must consist of one to four purely numeric parts separated by dots; anything else is a usage error.

## Version filter

- `--min-version` omits files whose version is lower than the given version.
- `--max-version` omits files whose version is higher than the given version.
- Both bounds are inclusive and may be combined to define a range.
- While a version filter is active, files whose version is not comparable are omitted silently.
- The version filter works independently of `--file-version`, which only controls whether the version column is shown.

## Output

- One line per file, written to stdout.
- Without `--file-version`: the full path of the file.
- With `--file-version`: the file version, left-aligned and padded with spaces to a width of 16 characters, followed by the full path.
  A version of 16 or more characters is followed by a single space.
  For files without a version the version column consists of spaces only.
- Lines end with CRLF.
- The output is encoded as UTF-8 without a byte order mark, on stdout and in the output file.
  When stdout is a Windows console, the text is shown correctly regardless of the console code page.
- With `--output` the same lines are written to stdout and to the output file.

Example (`flist "*.exe" --file-version --sort path --dir C:\Tools`):

```text
1.1.30.01       C:\Tools\AutoHotkey\BasicTextExpansion.exe
14.08           C:\Tools\Autoruns\autoruns.exe
                C:\Tools\Cheatbox\cheatbox.exe
1.0.0.0         C:\Tools\FileList\flist.exe
```

## Error handling

- Errors are reported as a single-line message on stderr, without a stack trace and without the help text.
- Warnings (inaccessible entries) are also written to stderr.

| Exit code | Meaning |
|---|---|
| `0` | Success. |
| `1` | Runtime error, e.g. the start directory does not exist, the output file cannot be written, or an inaccessible entry was skipped with a warning. |
| `2` | Usage error, e.g. unknown option, missing option value or invalid version argument. |
