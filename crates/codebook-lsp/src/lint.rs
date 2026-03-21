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

fn fatal(msg: impl std::fmt::Display) -> ! {
    eprintln!(
        "{} {msg}",
        "error:".if_supports_color(Stream::Stderr, |t| t.style(BOLD_RED))
    );
    std::process::exit(2);
}

/// Computes a workspace-relative path string for a given file. Falls back to
/// the absolute path if the file is outside the workspace or canonicalization fails.
/// `root_canonical` should be the already-canonicalized workspace root.
fn relative_to_root(root_canonical: Option<&Path>, path: &Path) -> String {
    let Some(root_canonical) = root_canonical else {
        return path.to_string_lossy().into_owned();
    };
    match path.canonicalize() {
        Ok(canon) => match canon.strip_prefix(root_canonical) {
            Ok(rel) => rel.to_string_lossy().into_owned(),
            Err(_) => path.to_string_lossy().into_owned(),
        },
        Err(_) => path.to_string_lossy().into_owned(),
    }
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

    let mut had_failure = false;
    let resolved = resolve_paths(files, root, &mut had_failure);

    let mut seen_words: HashSet<String> = HashSet::new();
    let mut total_errors = 0usize;
    let mut files_with_errors = 0usize;

    for path in &resolved {
        let relative = relative_to_root(root_canonical.as_deref(), path);
        let error_count = check_file(
            path,
            &relative,
            &codebook,
            &mut seen_words,
            unique,
            suggest,
            &mut had_failure,
        );
        if error_count > 0 {
            total_errors += error_count;
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
/// Returns the number of spelling errors found (0 if the file was clean).
/// `relative` is the workspace-relative path used for display and ignore matching.
/// Sets `*had_failure = true` on I/O errors so the caller can exit non-zero.
fn check_file(
    path: &Path,
    relative: &str,
    codebook: &Codebook,
    seen_words: &mut HashSet<String>,
    unique: bool,
    suggest: bool,
    had_failure: &mut bool,
) -> usize {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!(
                "{} {}: {e}",
                "error:".if_supports_color(Stream::Stderr, |t| t.style(BOLD_RED)),
                path.display()
            );
            *had_failure = true;
            return 0;
        }
    };

    let display = relative.strip_prefix("./").unwrap_or(relative);

    let mut locations = codebook.spell_check(&text, None, Some(relative));
    // Sort by first occurrence in the file.
    locations.sort_by_key(|l| l.locations.first().map(|r| r.start_byte).unwrap_or(0));

    // Build the offset table once per file – O(n) construction, O(1) per lookup –
    // and gives Unicode character columns rather than raw byte offsets.
    let offsets = StringOffsets::<AllConfig>::new(&text);

    // Collect (linecol, word, suggestions) first so we can compute pad_len for alignment.
    // The unique check is per-word (outer loop) so all ranges of a word are included
    // or skipped together.
    let mut hits: Vec<(String, &str, Option<Vec<String>>)> = Vec::new();
    for wl in &locations {
        if unique && !seen_words.insert(wl.word.to_lowercase()) {
            continue;
        }

        let suggestions = if suggest {
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

        for range in ranges {
            // utf8_to_char_pos returns 0-based line and Unicode-char column.
            let pos = offsets.utf8_to_char_pos(range.start_byte.min(text.len()));
            let line = pos.line + 1; // 0-based → 1-based
            let col = pos.col + 1; // 0-based → 1-based
            hits.push((
                format!("{line}:{col}"),
                wl.word.as_str(),
                suggestions.clone(),
            ));
        }
    }

    if hits.is_empty() {
        return 0;
    }

    let pad_len = hits
        .iter()
        .map(|(linecol, _, _)| linecol.len())
        .max()
        .unwrap_or(0);

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

    hits.len()
}

/// Prints which config file is being used, or notes that the default is active.
fn print_config_source(config: &CodebookConfigFile) {
    let cwd = std::env::current_dir().unwrap_or_default();
    match (
        config.project_config_path().filter(|p| p.is_file()),
        config.global_config_path().filter(|p| p.is_file()),
    ) {
        (Some(p), _) => {
            let path = p.strip_prefix(&cwd).unwrap_or(&p).display().to_string();
            eprintln!(
                "using config {}",
                path.if_supports_color(Stream::Stderr, |t| t.style(DIM))
            );
        }
        (None, Some(g)) => {
            let path = g.strip_prefix(&cwd).unwrap_or(&g).display().to_string();
            eprintln!(
                "using global config {}",
                path.if_supports_color(Stream::Stderr, |t| t.style(DIM))
            );
        }
        (None, None) => eprintln!("No config found, using default config"),
    }
}

/// Resolves a mix of file paths, directories, and glob patterns into a sorted,
/// deduplicated list of file paths. Non-absolute patterns are resolved relative to `root`.
///
/// Sets `*had_failure = true` for unmatched patterns, invalid globs, or glob I/O errors.
fn resolve_paths(patterns: &[String], root: &Path, had_failure: &mut bool) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for pattern in patterns {
        let p = PathBuf::from(pattern);
        let p = if p.is_absolute() { p } else { root.join(&p) };
        if p.is_dir() {
            collect_dir(&p, &mut paths, had_failure);
        } else {
            let pattern_str = p.to_string_lossy();
            match glob::glob(&pattern_str) {
                Ok(entries) => {
                    let mut matched = false;
                    for entry in entries {
                        match entry {
                            Ok(e) => {
                                if e.is_file() {
                                    paths.push(e);
                                    matched = true;
                                } else if e.is_dir() {
                                    collect_dir(&e, &mut paths, had_failure);
                                    matched = true;
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "{} failed to read glob entry: {e}",
                                    "error:"
                                        .if_supports_color(Stream::Stderr, |t| t.style(BOLD_RED))
                                );
                                *had_failure = true;
                            }
                        }
                    }
                    if !matched {
                        eprintln!(
                            "{} no match for '{pattern_str}'",
                            "error:".if_supports_color(Stream::Stderr, |t| t.style(BOLD_RED))
                        );
                        *had_failure = true;
                    }
                }
                Err(e) => {
                    eprintln!(
                        "{} invalid pattern '{pattern_str}': {e}",
                        "error:".if_supports_color(Stream::Stderr, |t| t.style(BOLD_RED))
                    );
                    *had_failure = true;
                }
            }
        }
    }
    paths.sort();
    paths.dedup();
    paths
}

/// Recursively collects all files under `dir` into `out`.
/// Sets `*had_failure = true` for any directory-entry I/O error (e.g. permission denied).
fn collect_dir(dir: &Path, out: &mut Vec<PathBuf>, had_failure: &mut bool) {
    for entry in walkdir::WalkDir::new(dir).follow_links(false).into_iter() {
        match entry {
            Ok(e) => {
                if e.file_type().is_file() {
                    out.push(e.into_path());
                }
            }
            Err(err) => {
                eprintln!(
                    "{} failed to read directory entry under '{}': {err}",
                    "error:".if_supports_color(Stream::Stderr, |t| t.style(BOLD_RED)),
                    dir.display()
                );
                *had_failure = true;
            }
        }
    }
}
