use codebook::Codebook;
use codebook_config::CodebookConfigFile;
use owo_colors::{OwoColorize, Stream, Style};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use string_offsets::{AllConfig, StringOffsets};

const BOLD: Style = Style::new().bold();
const DIM: Style = Style::new().dimmed();
const YELLOW: Style = Style::new().yellow();
const BOLD_RED: Style = Style::new().bold().red();

macro_rules! err {
    ($($arg:tt)*) => {
        eprintln!(
            "{} {}",
            "error:".if_supports_color(Stream::Stderr, |t| t.style(BOLD_RED)),
            format_args!($($arg)*)
        )
    };
}

fn fatal(msg: impl std::fmt::Display) -> ! {
    err!("{msg}");
    std::process::exit(2);
}

/// Computes a workspace-relative path string for a given file. Falls back to
/// the absolute path if the file is outside the workspace or canonicalization fails.
/// `root_canonical` should be the already-canonicalized workspace root.
fn relative_to_root(root_canonical: Option<&Path>, path: &Path) -> String {
    root_canonical
        .and_then(|root| {
            let canon = path.canonicalize().ok()?;
            canon
                .strip_prefix(root)
                .ok()
                .map(|rel| rel.to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}

/// Returns `true` if any spelling errors were found.
///
/// Exits with code 2 if infrastructure failures occurred (unreadable files,
/// directory errors, unmatched or invalid patterns).
pub fn run_lint(files: &[String], root: &Path, unique: bool, suggest: bool) -> bool {
    let config = Arc::new(
        CodebookConfigFile::load(Some(root))
            .unwrap_or_else(|e| fatal(format!("failed to load config: {e}"))),
    );

    print_config_source(&config);
    eprintln!();

    let codebook = Codebook::new(config.clone())
        .unwrap_or_else(|e| fatal(format!("failed to initialize: {e}")));

    // Canonicalize the root once here rather than once per file.
    let root_canonical = root.canonicalize().ok();

    let (resolved, mut had_failure) = resolve_paths(files, root);

    let mut seen_words: HashSet<String> = HashSet::new();
    let mut total_errors = 0usize;
    let mut files_with_errors = 0usize;

    for path in &resolved {
        let relative = relative_to_root(root_canonical.as_deref(), path);
        let (errors, file_failure) =
            check_file(path, &relative, &codebook, &mut seen_words, unique, suggest);
        had_failure |= file_failure;
        if errors > 0 {
            total_errors += errors;
            files_with_errors += 1;
        }
    }

    let unique_label = if unique { "unique " } else { "" };
    eprintln!(
        "Found {} {unique_label}spelling error(s) in {} file(s).",
        total_errors.if_supports_color(Stream::Stderr, |t| t.style(BOLD)),
        files_with_errors.if_supports_color(Stream::Stderr, |t| t.style(BOLD)),
    );

    if had_failure {
        std::process::exit(2);
    }

    total_errors > 0
}

/// Spell-checks a single file and prints any diagnostics to stdout.
///
/// Returns `(error_count, had_io_error)`. `error_count` is 0 if the file was
/// clean; `had_io_error` is true when the file could not be read.
/// `relative` is the workspace-relative path used for display and ignore matching.
fn check_file(
    path: &Path,
    relative: &str,
    codebook: &Codebook,
    seen_words: &mut HashSet<String>,
    unique: bool,
    suggest: bool,
) -> (usize, bool) {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            err!("{}: {e}", path.display());
            return (0, true);
        }
    };

    let display = relative.strip_prefix("./").unwrap_or(relative);

    // Build the offset table once per file – O(n) construction, O(1) per lookup –
    // and gives Unicode character columns rather than raw byte offsets.
    let offsets = StringOffsets::<AllConfig>::new(&text);

    let mut locations = codebook.spell_check(&text, None, Some(relative));
    // Sort by first occurrence in the file.
    locations.sort_by_key(|l| l.locations.first().map(|r| r.start_byte).unwrap_or(0));

    // Collect hits first so we can compute pad_len for column alignment.
    // The unique check is per-word (outer loop) so all ranges of a word are
    // included or skipped together.
    let mut hits: Vec<(String, &str, Option<Vec<String>>)> = Vec::new();
    for wl in &locations {
        if unique && !seen_words.insert(wl.word.to_lowercase()) {
            continue;
        }

        let mut suggestions = if suggest {
            codebook.get_suggestions(wl.word.as_str())
        } else {
            None
        };

        // In unique mode only emit the first occurrence of each word.
        let ranges = if unique {
            &wl.locations[..1]
        } else {
            &wl.locations[..]
        };

        for (i, range) in ranges.iter().enumerate() {
            // utf8_to_char_pos returns 0-based line and Unicode-char column.
            let pos = offsets.utf8_to_char_pos(range.start_byte.min(text.len()));
            // Move out of `suggestions` on the last iteration to avoid a clone.
            let sugg = if i + 1 < ranges.len() {
                suggestions.clone()
            } else {
                suggestions.take()
            };
            hits.push((
                format!("{}:{}", pos.line + 1, pos.col + 1),
                wl.word.as_str(),
                sugg,
            ));
        }
    }

    if hits.is_empty() {
        return (0, false);
    }

    let pad_len = hits.iter().map(|(lc, _, _)| lc.len()).max().unwrap_or(0);

    println!(
        "{}",
        display.if_supports_color(Stream::Stdout, |t| t.style(BOLD))
    );
    for (linecol, word, suggestions) in &hits {
        let pad = " ".repeat(pad_len - linecol.len());
        print!(
            "  {}:{}{}  {}",
            display.if_supports_color(Stream::Stdout, |t| t.style(DIM)),
            linecol.if_supports_color(Stream::Stdout, |t| t.style(YELLOW)),
            pad,
            word.if_supports_color(Stream::Stdout, |t| t.style(BOLD_RED)),
        );
        if let Some(suggestions) = suggestions {
            println!(
                "  {}",
                format!("→ {}", suggestions.join(", "))
                    .if_supports_color(Stream::Stdout, |t| t.style(DIM)),
            );
        } else {
            println!();
        }
    }
    println!();

    (hits.len(), false)
}

/// Prints which config file is being used, or notes that the default is active.
fn print_config_source(config: &CodebookConfigFile) {
    let cwd = std::env::current_dir().unwrap_or_default();
    let (label, path) = match (
        config.project_config_path().filter(|p| p.is_file()),
        config.global_config_path().filter(|p| p.is_file()),
    ) {
        (Some(p), _) => ("using config", p),
        (None, Some(g)) => ("using global config", g),
        (None, None) => {
            eprintln!("No config found, using default config");
            return;
        }
    };
    let display = path
        .strip_prefix(&cwd)
        .unwrap_or(&path)
        .display()
        .to_string();
    eprintln!(
        "{label} {}",
        display.if_supports_color(Stream::Stderr, |t| t.style(DIM))
    );
}

/// Resolves a mix of file paths, directories, and glob patterns into a sorted,
/// deduplicated list of file paths. Non-absolute patterns are resolved relative
/// to `root`. `Path::join` replaces the base when the argument is absolute, so
/// no explicit `is_absolute` check is needed.
///
/// Returns `(paths, had_failure)`. `had_failure` is true for unmatched
/// patterns, invalid globs, or glob I/O errors.
fn resolve_paths(patterns: &[String], root: &Path) -> (Vec<PathBuf>, bool) {
    let mut paths = Vec::new();
    let mut had_failure = false;

    for pattern in patterns {
        // root.join() is a no-op when pattern is absolute (replaces root entirely).
        let p = root.join(pattern);
        if p.is_dir() {
            had_failure |= collect_dir(&p, &mut paths);
        } else {
            let pattern_str = p.to_string_lossy();
            match glob::glob(&pattern_str) {
                Ok(entries) => {
                    let mut matched = false;
                    for entry in entries {
                        match entry {
                            Ok(e) if e.is_file() => {
                                paths.push(e);
                                matched = true;
                            }
                            Ok(e) if e.is_dir() => {
                                had_failure |= collect_dir(&e, &mut paths);
                                matched = true;
                            }
                            Ok(_) => {}
                            Err(e) => {
                                err!("failed to read glob entry: {e}");
                                had_failure = true;
                            }
                        }
                    }
                    if !matched {
                        err!("no match for '{pattern_str}'");
                        had_failure = true;
                    }
                }
                Err(e) => {
                    err!("invalid pattern '{pattern_str}': {e}");
                    had_failure = true;
                }
            }
        }
    }

    paths.sort();
    paths.dedup();
    (paths, had_failure)
}

/// Recursively collects all files under `dir` into `out`.
/// Returns `true` if any directory-entry I/O error occurred (e.g. permission denied).
fn collect_dir(dir: &Path, out: &mut Vec<PathBuf>) -> bool {
    let mut had_failure = false;
    for entry in walkdir::WalkDir::new(dir).follow_links(false) {
        match entry {
            Ok(e) if e.file_type().is_file() => out.push(e.into_path()),
            Ok(_) => {}
            Err(e) => {
                err!(
                    "failed to read directory entry under '{}': {e}",
                    dir.display()
                );
                had_failure = true;
            }
        }
    }
    had_failure
}

#[cfg(test)]
mod tests {
    use super::*;
    use codebook::Codebook;
    use codebook_config::CodebookConfigMemory;
    use std::collections::HashSet;
    use std::fs;
    use std::sync::Arc;
    use tempfile::tempdir;

    /// Builds a Codebook backed by the default in-memory config (en_us dictionary).
    /// This mirrors the pattern used throughout the rest of the test suite.
    fn make_codebook() -> Codebook {
        let config = Arc::new(CodebookConfigMemory::default());
        Codebook::new(config).unwrap()
    }

    // ── relative_to_root ─────────────────────────────────────────────────────

    #[test]
    fn test_relative_to_root_inside_workspace() {
        let dir = tempdir().unwrap();
        let subdir = dir.path().join("src");
        fs::create_dir_all(&subdir).unwrap();
        let file = subdir.join("lib.rs");
        fs::write(&file, "").unwrap();

        let root_canonical = dir.path().canonicalize().unwrap();
        let result = relative_to_root(Some(&root_canonical), &file);
        assert_eq!(result, "src/lib.rs");
    }

    #[test]
    fn test_relative_to_root_outside_workspace() {
        let root = tempdir().unwrap();
        let other = tempdir().unwrap();
        let file = other.path().join("outside.rs");
        fs::write(&file, "").unwrap();

        let root_canonical = root.path().canonicalize().unwrap();
        let result = relative_to_root(Some(&root_canonical), &file);
        // Falls back to a path string since the file is outside the workspace.
        assert!(result.contains("outside.rs"));
    }

    #[test]
    fn test_relative_to_root_none_returns_raw_path() {
        let result = relative_to_root(None, Path::new("/some/path/file.rs"));
        assert_eq!(result, "/some/path/file.rs");
    }

    // ── collect_dir ───────────────────────────────────────────────────────────

    #[test]
    fn test_collect_dir_empty_dir() {
        let dir = tempdir().unwrap();
        let mut out = Vec::new();
        let had_failure = collect_dir(dir.path(), &mut out);
        assert!(out.is_empty());
        assert!(!had_failure);
    }

    #[test]
    fn test_collect_dir_collects_files_recursively() {
        let dir = tempdir().unwrap();
        let sub = dir.path().join("sub");
        fs::create_dir_all(&sub).unwrap();
        fs::write(dir.path().join("a.txt"), "").unwrap();
        fs::write(sub.join("b.txt"), "").unwrap();

        let mut out = Vec::new();
        let had_failure = collect_dir(dir.path(), &mut out);

        assert_eq!(out.len(), 2);
        assert!(!had_failure);
    }

    #[test]
    fn test_collect_dir_only_yields_files_not_directories() {
        let dir = tempdir().unwrap();
        let sub = dir.path().join("subdir");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("c.txt"), "").unwrap();

        let mut out = Vec::new();
        let had_failure = collect_dir(dir.path(), &mut out);

        assert!(
            out.iter().all(|p| p.is_file()),
            "only files should be collected, not directories"
        );
        assert!(!had_failure);
    }

    // ── resolve_paths ─────────────────────────────────────────────────────────

    #[test]
    fn test_resolve_paths_single_existing_file() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("hello.txt");
        fs::write(&file, "").unwrap();

        let (result, had_failure) =
            resolve_paths(&[file.to_string_lossy().to_string()], dir.path());

        assert_eq!(result, vec![file]);
        assert!(!had_failure);
    }

    #[test]
    fn test_resolve_paths_missing_file_sets_failure() {
        let dir = tempdir().unwrap();
        let nonexistent = dir.path().join("nope.txt");

        let (result, had_failure) =
            resolve_paths(&[nonexistent.to_string_lossy().to_string()], dir.path());

        assert!(result.is_empty());
        assert!(had_failure, "unmatched path should set had_failure");
    }

    #[test]
    fn test_resolve_paths_directory_collects_all_files() {
        let dir = tempdir().unwrap();
        let sub = dir.path().join("sub");
        fs::create_dir_all(&sub).unwrap();
        fs::write(dir.path().join("a.txt"), "").unwrap();
        fs::write(sub.join("b.txt"), "").unwrap();

        let (result, had_failure) =
            resolve_paths(&[dir.path().to_string_lossy().to_string()], dir.path());

        assert_eq!(result.len(), 2);
        assert!(!had_failure);
    }

    #[test]
    fn test_resolve_paths_glob_matches_by_extension() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("foo.rs"), "").unwrap();
        fs::write(dir.path().join("bar.rs"), "").unwrap();
        fs::write(dir.path().join("baz.txt"), "").unwrap();

        let pattern = format!("{}/*.rs", dir.path().to_string_lossy());
        let (result, had_failure) = resolve_paths(&[pattern], dir.path());

        assert_eq!(result.len(), 2, "only .rs files should match");
        assert!(
            result
                .iter()
                .all(|p| p.extension().unwrap_or_default() == "rs")
        );
        assert!(!had_failure);
    }

    #[test]
    fn test_resolve_paths_glob_no_match_sets_failure() {
        let dir = tempdir().unwrap(); // empty dir — no .rs files
        let pattern = format!("{}/*.rs", dir.path().to_string_lossy());

        let (result, had_failure) = resolve_paths(&[pattern], dir.path());

        assert!(result.is_empty());
        assert!(had_failure, "unmatched glob should set had_failure");
    }

    #[test]
    fn test_resolve_paths_deduplicates_same_file() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("hello.txt");
        fs::write(&file, "").unwrap();

        let path_str = file.to_string_lossy().to_string();
        // List the same file twice — it should appear only once in the output.
        let (result, had_failure) = resolve_paths(&[path_str.clone(), path_str], dir.path());

        assert_eq!(result.len(), 1);
        assert!(!had_failure);
    }

    #[test]
    fn test_resolve_paths_output_is_sorted() {
        let dir = tempdir().unwrap();
        // Create files in non-alphabetical order.
        fs::write(dir.path().join("c.txt"), "").unwrap();
        fs::write(dir.path().join("a.txt"), "").unwrap();
        fs::write(dir.path().join("b.txt"), "").unwrap();

        let (result, had_failure) =
            resolve_paths(&[dir.path().to_string_lossy().to_string()], dir.path());

        let names: Vec<_> = result
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["a.txt", "b.txt", "c.txt"]);
        assert!(!had_failure);
    }

    #[test]
    fn test_resolve_paths_relative_pattern_resolved_against_root() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("x.txt"), "").unwrap();

        // "x.txt" is relative — resolve_paths should join it with the root.
        let (result, had_failure) = resolve_paths(&["x.txt".to_string()], dir.path());

        assert_eq!(result.len(), 1);
        assert!(!had_failure);
    }

    // ── check_file ────────────────────────────────────────────────────────────

    #[test]
    fn test_check_file_unreadable_sets_failure() {
        let dir = tempdir().unwrap();
        let nonexistent = dir.path().join("ghost.txt");

        let codebook = make_codebook();
        let (count, had_failure) = check_file(
            &nonexistent,
            "ghost.txt",
            &codebook,
            &mut HashSet::new(),
            false,
            false,
        );

        assert_eq!(count, 0);
        assert!(had_failure, "unreadable file must set had_failure");
    }

    #[test]
    fn test_check_file_flags_misspelling() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("bad.txt");
        // "actualbad" is not a real English word; the checker should flag it.
        fs::write(&file, "actualbad").unwrap();

        let codebook = make_codebook();
        let (count, had_failure) = check_file(
            &file,
            "bad.txt",
            &codebook,
            &mut HashSet::new(),
            false,
            false,
        );

        assert!(
            count > 0,
            "expected at least one misspelling to be reported"
        );
        assert!(!had_failure);
    }

    #[test]
    fn test_check_file_unique_counts_each_word_once_per_file() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("dup.txt");
        // The checker groups all occurrences of the same misspelled word into one
        // WordLocation with multiple TextRanges; unique mode takes only the first.
        fs::write(&file, "actualbad actualbad").unwrap();

        let codebook = make_codebook();
        let mut had_failure = false;

        let (count_all, f) = check_file(
            &file,
            "dup.txt",
            &codebook,
            &mut HashSet::new(),
            false, // unique = false
            false,
        );
        had_failure |= f;

        let (count_unique, f) = check_file(
            &file,
            "dup.txt",
            &codebook,
            &mut HashSet::new(),
            true, // unique = true
            false,
        );
        had_failure |= f;

        assert_eq!(count_all, 2, "non-unique mode must count every occurrence");
        assert_eq!(count_unique, 1, "unique mode must count each word once");
        assert!(!had_failure);
    }

    #[test]
    fn test_check_file_unique_deduplicates_across_files() {
        let dir = tempdir().unwrap();
        let file1 = dir.path().join("first.txt");
        let file2 = dir.path().join("second.txt");
        fs::write(&file1, "actualbad").unwrap();
        fs::write(&file2, "actualbad").unwrap();

        let codebook = make_codebook();
        // A single shared `seen_words` set, exactly as run_lint uses it.
        let mut seen = HashSet::new();

        let (count1, f1) = check_file(&file1, "first.txt", &codebook, &mut seen, true, false);
        let (count2, f2) = check_file(&file2, "second.txt", &codebook, &mut seen, true, false);

        assert_eq!(count1, 1, "first file should report the misspelling");
        assert_eq!(count2, 0, "second file should skip the already-seen word");
        assert!(!f1);
        assert!(!f2);
    }

    // ── line/col correctness (pure StringOffsets, no network I/O) ────────────
    //
    // These tests exercise the exact same utf8_to_char_pos call that check_file
    // uses, confirming that columns are in Unicode characters rather than raw
    // bytes and that the 0-based → 1-based conversion is applied correctly.

    #[test]
    fn test_linecol_word_at_start_of_file() {
        let text = "actualbad rest of line";
        let offsets = StringOffsets::<AllConfig>::new(text);
        let pos = offsets.utf8_to_char_pos(0);
        assert_eq!(pos.line + 1, 1);
        assert_eq!(pos.col + 1, 1);
    }

    #[test]
    fn test_linecol_word_on_second_line() {
        // "ok text\n" is 8 bytes; "actualbad" starts at byte 8.
        let text = "ok text\nactualbad more";
        let offsets = StringOffsets::<AllConfig>::new(text);
        let byte_offset = text.find("actualbad").unwrap();
        assert_eq!(byte_offset, 8, "sanity-check byte layout");

        let pos = offsets.utf8_to_char_pos(byte_offset);
        assert_eq!(pos.line + 1, 2, "should be on the second line");
        assert_eq!(pos.col + 1, 1, "should be the first column of that line");
    }

    #[test]
    fn test_linecol_mid_line_ascii() {
        // "hello " is 6 bytes/chars; "actualbad" starts at byte/char 6.
        let text = "hello actualbad";
        let offsets = StringOffsets::<AllConfig>::new(text);
        let byte_offset = text.find("actualbad").unwrap();
        assert_eq!(byte_offset, 6, "sanity-check byte layout");

        let pos = offsets.utf8_to_char_pos(byte_offset);
        assert_eq!(pos.line + 1, 1);
        assert_eq!(pos.col + 1, 7);
    }

    /// The old byte-count approach reported col=10 here; StringOffsets correctly
    /// reports col=8 because "résumé" is 8 bytes but only 6 Unicode characters.
    #[test]
    fn test_linecol_multibyte_chars_before_word() {
        // r(1) + é(2) + s(1) + u(1) + m(1) + é(2) + space(1) = 9 bytes, 7 chars.
        // "actualbad" starts at byte 9, char offset 7 (0-based) → col 8 (1-based).
        let text = "résumé actualbad";
        let offsets = StringOffsets::<AllConfig>::new(text);
        let byte_offset = text.find("actualbad").unwrap();
        assert_eq!(byte_offset, 9, "sanity-check byte layout");

        let pos = offsets.utf8_to_char_pos(byte_offset);
        assert_eq!(pos.line + 1, 1);
        assert_eq!(
            pos.col + 1,
            8,
            "col must count Unicode chars (6 + space = 7, 1-based = 8), not bytes (9, 1-based = 10)"
        );
    }

    #[test]
    fn test_linecol_emoji_before_word() {
        // 🐛 is U+1F41B — 4 UTF-8 bytes, 1 Unicode char.
        // "🐛 " = 5 bytes, 2 chars; "actualbad" starts at byte 5, char 2 (0-based) → col 3.
        let text = "🐛 actualbad";
        let offsets = StringOffsets::<AllConfig>::new(text);
        let byte_offset = text.find("actualbad").unwrap();
        assert_eq!(byte_offset, 5, "sanity-check byte layout");

        let pos = offsets.utf8_to_char_pos(byte_offset);
        assert_eq!(pos.line + 1, 1);
        assert_eq!(
            pos.col + 1,
            3,
            "col must count Unicode chars (emoji=1, space=1, 1-based=3), not bytes (5, 1-based=6)"
        );
    }
}
