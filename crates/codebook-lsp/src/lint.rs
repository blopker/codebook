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

macro_rules! paint {
    ($val:expr, $stream:expr, $style:expr) => {
        $val.if_supports_color($stream, |t| t.style($style))
    };
}

fn fatal(msg: impl std::fmt::Display) -> ! {
    err!("{msg}");
    std::process::exit(2);
}

/// Computes a workspace-relative path string for a given file. Falls back to
/// the absolute path if the file is outside the workspace or canonicalization
/// fails. `root_canonical` should be the already-canonicalized workspace root.
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
        paint!(total_errors, Stream::Stderr, BOLD),
        paint!(files_with_errors, Stream::Stderr, BOLD),
    );

    if had_failure {
        std::process::exit(2);
    }

    total_errors > 0
}

/// Spell-checks a single file and prints any diagnostics to stdout.
///
/// Returns `(error_count, had_io_error)`. `error_count` is 0 if the file was
/// clean; `had_io_error` is true when the file could not be read. `relative` is
/// the workspace-relative path used for display and ignore matching.
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

    // Build the offset table once per file
    let offsets = StringOffsets::<AllConfig>::new(&text);
    let mut locations = codebook.spell_check(&text, None, Some(relative));
    // Sort by first occurrence in the file.
    locations.sort_by_key(|l| l.locations.first().map(|r| r.start_byte).unwrap_or(0));

    // Collect hits first so we can compute pad_len for column alignment. The
    // unique check is per-word, so all ranges of a word are included or skipped
    // together.
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

        // If unique mode: Only emit the first occurrence of each word.
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
            paint!(display, Stream::Stdout, DIM),
            paint!(linecol, Stream::Stdout, YELLOW),
            pad,
            paint!(word, Stream::Stdout, BOLD_RED),
        );
        if let Some(s) = suggestions {
            let text = format!("→ {}", s.join(", "));
            println!("  {}", paint!(text, Stream::Stdout, DIM));
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
        // root.join() is a no-op when pattern is absolute
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

/// Recursively collects all files under `dir` into `out`. Returns `true` if any
/// directory-entry I/O error occurred.
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

    #[test]
    fn test_path_and_dir_resolution() {
        let dir = tempdir().unwrap();
        let sub = dir.path().join("sub");
        fs::create_dir_all(&sub).unwrap();

        let f1 = dir.path().join("a.rs");
        let f2 = sub.join("b.txt");
        fs::write(&f1, "").unwrap();
        fs::write(&f2, "").unwrap();

        let root_canon = dir.path().canonicalize().unwrap();
        assert_eq!(relative_to_root(Some(&root_canon), &f1), "a.rs");

        let pattern = format!("{}/**/*.*", dir.path().display());
        let (paths, err) = resolve_paths(&[pattern], dir.path());

        assert!(!err);
        assert_eq!(paths.len(), 2);
        let path_strs: HashSet<_> = paths.iter().map(|p| p.to_string_lossy()).collect();
        assert!(path_strs.iter().any(|s| s.ends_with("a.rs")));
        assert!(path_strs.iter().any(|s| s.ends_with("b.txt")));

        let (_, err_missing) = resolve_paths(&["nonexistent.rs".into()], dir.path());
        assert!(err_missing);
    }

    #[test]
    fn test_check_file_logic() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("test.txt");
        fs::write(&f, "actualbad\n🦀 actualbad").unwrap();

        let cb = Codebook::new(Arc::new(CodebookConfigMemory::default())).unwrap();
        let mut seen = HashSet::new();

        // Test basic flagging and multi-occurrence counting
        let (count, err) = check_file(&f, "test.txt", &cb, &mut seen, false, false);
        assert_eq!(count, 2);
        assert!(!err);

        // Test unique mode
        let mut seen_unique = HashSet::new();
        let (c1, _) = check_file(&f, "f1.txt", &cb, &mut seen_unique, true, false);
        let (c2, _) = check_file(&f, "f2.txt", &cb, &mut seen_unique, true, false);
        assert_eq!(c1, 1, "Should flag word once");
        assert_eq!(c2, 0, "Should skip already-seen word in second file");

        // Test IO failure
        let (_, err_io) = check_file(
            &dir.path().join("missing"),
            "!",
            &cb,
            &mut seen,
            false,
            false,
        );
        assert!(err_io);
    }

    #[test]
    fn test_unicode_line_col() {
        let cases = [
            ("actualbad", 0, 1, 1),        // Start
            ("ok\nactualbad", 3, 2, 1),    // Newline
            ("résumé actualbad", 9, 1, 8), // Multi-byte chars (é is 2 bytes)
            ("🦀 actualbad", 5, 1, 3),     // Emoji (4 bytes, 1 char)
        ];

        for (text, offset, line, col) in cases {
            let table = StringOffsets::<AllConfig>::new(text);
            let pos = table.utf8_to_char_pos(offset);
            assert_eq!(
                (pos.line + 1, pos.col + 1),
                (line, col),
                "Failed on: {}",
                text
            );
        }
    }
}
