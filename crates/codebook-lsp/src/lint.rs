use codebook::Codebook;
use codebook_config::{CodebookConfig, CodebookConfigFile};
use ignore::WalkBuilder;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use string_offsets::{AllConfig, StringOffsets};

macro_rules! err {
    ($($arg:tt)*) => {
        eprintln!("error: {}", format_args!($($arg)*))
    };
}

/// Result of a lint run, mapped to exit codes by the caller.
pub enum LintResult {
    /// All files clean — exit 0.
    Clean,
    /// Spelling errors found — exit 1.
    Errors,
    /// Infrastructure failure (IO errors, bad patterns, etc.) — exit 2.
    Failure,
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

pub fn run_lint(files: &[String], root: &Path, unique: bool, suggest: bool) -> LintResult {
    let config = match CodebookConfigFile::load(Some(root)) {
        Ok(c) => Arc::new(c),
        Err(e) => {
            err!("failed to load config: {e}");
            return LintResult::Failure;
        }
    };

    print_config_source(&config);
    eprintln!();

    let codebook = match Codebook::new(config.clone()) {
        Ok(c) => c,
        Err(e) => {
            err!("failed to initialize: {e}");
            return LintResult::Failure;
        }
    };

    // Canonicalize the root once here rather than once per file.
    let root_canonical = root.canonicalize().ok();

    let (resolved, mut had_failure) = resolve_paths(files, root);

    let mut seen_words: HashSet<String> = HashSet::new();
    let mut total_errors = 0usize;
    let mut files_with_errors = 0usize;

    for path in &resolved {
        let relative = relative_to_root(root_canonical.as_deref(), path);

        if config.should_ignore_path(Path::new(&relative)) {
            continue;
        }
        if !config.should_include_path(Path::new(&relative)) {
            continue;
        }

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
        "Found {total_errors} {unique_label}spelling error(s) in {files_with_errors} file(s)."
    );

    if had_failure {
        LintResult::Failure
    } else if total_errors > 0 {
        LintResult::Errors
    } else {
        LintResult::Clean
    }
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
        Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {
            // Binary / non-UTF-8 file — silently skip.
            return (0, false);
        }
        Err(e) => {
            err!("{}: {e}", path.display());
            return (0, true);
        }
    };

    let display = relative.strip_prefix("./").unwrap_or(relative);

    // Build the offset table once per file
    let offsets = StringOffsets::<AllConfig>::new(&text);
    let mut locations = codebook.spell_check(&text, None, Some(relative));
    // Sort inner locations first (HashSet iteration order is nondeterministic),
    // then sort the outer list by first occurrence in the file.
    for wl in &mut locations {
        wl.locations.sort_by_key(|r| r.start_byte);
    }
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

    println!("{display}");
    for (linecol, word, suggestions) in &hits {
        let pad = " ".repeat(pad_len - linecol.len());
        if let Some(s) = suggestions {
            println!("  {display}:{linecol}{pad}  {word}  -> {}", s.join(", "));
        } else {
            println!("  {display}:{linecol}{pad}  {word}");
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
    eprintln!("{label} {display}");
}

/// Builds a gitignore matcher from the `.gitignore` file in `root`, if present.
fn build_gitignore(root: &Path) -> Gitignore {
    let mut builder = GitignoreBuilder::new(root);
    builder.add(root.join(".gitignore"));
    builder.build().unwrap_or_else(|_| Gitignore::empty())
}

/// Resolves a mix of file paths, directories, and glob patterns into a sorted,
/// deduplicated list of file paths. Directories are walked using the `ignore`
/// crate, which automatically respects `.gitignore` rules and skips hidden
/// files/directories. Glob-matched files are also filtered against `.gitignore`.
///
/// Returns `(paths, had_failure)`. `had_failure` is true for unmatched
/// patterns, invalid globs, or walk I/O errors.
fn resolve_paths(patterns: &[String], root: &Path) -> (Vec<PathBuf>, bool) {
    let mut paths = Vec::new();
    let mut had_failure = false;
    let gitignore = build_gitignore(root);

    for pattern in patterns {
        // root.join() is a no-op when pattern is absolute
        let p = root.join(pattern);
        if p.is_dir() {
            had_failure |= collect_dir(&p, &mut paths);
        } else if p.is_file() {
            paths.push(p);
        } else {
            // Try as a glob pattern
            let pattern_str = p.to_string_lossy();
            match glob::glob(&pattern_str) {
                Ok(entries) => {
                    let mut matched = false;
                    for entry in entries {
                        match entry {
                            Ok(e) if e.is_file() => {
                                if !gitignore.matched(&e, false).is_ignore() {
                                    paths.push(e);
                                }
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

/// Recursively collects all files under `dir` into `out`, respecting
/// `.gitignore` rules and skipping hidden files/directories.
/// Returns `true` if any I/O error occurred during the walk.
fn collect_dir(dir: &Path, out: &mut Vec<PathBuf>) -> bool {
    let mut had_failure = false;
    for entry in WalkBuilder::new(dir).follow_links(false).build() {
        match entry {
            Ok(e) if e.file_type().is_some_and(|ft| ft.is_file()) => out.push(e.into_path()),
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
