use codebook::Codebook;
use codebook_config::CodebookConfigFile;
use owo_colors::{OwoColorize, Stream, Style};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

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
fn relative_to_root(root: &Path, path: &Path) -> String {
    let root_canonical = match root.canonicalize() {
        Ok(r) => r,
        Err(_) => return path.to_string_lossy().to_string(),
    };
    match path.canonicalize() {
        Ok(canon) => match canon.strip_prefix(&root_canonical) {
            Ok(rel) => rel.to_string_lossy().to_string(),
            Err(_) => path.to_string_lossy().to_string(),
        },
        Err(_) => path.to_string_lossy().to_string(),
    }
}

/// Returns `true` if any spelling errors were found.
pub fn run_lint(files: &[String], root: &Path, unique: bool, suggest: bool) -> bool {
    let config = Arc::new(
        CodebookConfigFile::load(Some(root))
            .unwrap_or_else(|e| fatal(format!("failed to load config: {e}"))),
    );

    print_config_source(&config);
    eprintln!();

    let codebook = Codebook::new(config.clone())
        .unwrap_or_else(|e| fatal(format!("failed to initialize: {e}")));

    let resolved = resolve_paths(files, root);

    let mut seen_words: HashSet<String> = HashSet::new();
    let mut total_errors = 0usize;
    let mut files_with_errors = 0usize;

    for path in &resolved {
        let relative = relative_to_root(root, path);
        let error_count = check_file(path, &relative, &codebook, &mut seen_words, unique, suggest);
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

    total_errors > 0
}

/// Spell-checks a single file and prints any diagnostics to stdout.
///
/// Returns the number of errors found (0 if the file was clean or unreadable).
/// `relative` is the workspace-relative path used for display and ignore matching.
fn check_file(
    path: &Path,
    relative: &str,
    codebook: &Codebook,
    seen_words: &mut HashSet<String>,
    unique: bool,
    suggest: bool,
) -> usize {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!(
                "{} {}: {e}",
                "error:".if_supports_color(Stream::Stderr, |t| t.style(BOLD_RED)),
                path.display()
            );
            return 0;
        }
    };

    let display = relative.strip_prefix("./").unwrap_or(relative);

    let mut locations = codebook.spell_check(&text, None, Some(relative));
    // Sort by first occurrence in the file.
    locations.sort_by_key(|l| l.locations.first().map(|r| r.start_byte).unwrap_or(0));

    // Collect (linecol, word, suggestions) first so we can compute pad_len for alignment.
    // unique check is per-word (outer loop) so all ranges of a word are included or skipped together.
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

        // In unique mode only emit the first occurrence of each word
        let ranges = if unique {
            &wl.locations[..1]
        } else {
            &wl.locations[..]
        };

        for range in ranges {
            let before = &text[..range.start_byte.min(text.len())];
            let line = before.bytes().filter(|&b| b == b'\n').count() + 1;
            let col = before
                .rfind('\n')
                .map(|p| before.len() - p)
                .unwrap_or(before.len() + 1);
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
/// deduplicated list of file paths. Non-absolute patterns are resolved relative to root.
fn resolve_paths(patterns: &[String], root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for pattern in patterns {
        let p = PathBuf::from(pattern);
        let p = if p.is_absolute() { p } else { root.join(&p) };
        if p.is_dir() {
            collect_dir(&p, &mut paths);
        } else {
            let pattern = p.to_string_lossy();
            match glob::glob(&pattern) {
                Ok(entries) => {
                    let mut matched = false;
                    for entry in entries.flatten() {
                        if entry.is_file() {
                            paths.push(entry);
                            matched = true;
                        } else if entry.is_dir() {
                            collect_dir(&entry, &mut paths);
                            matched = true;
                        }
                    }
                    if !matched {
                        eprintln!("codebook: no match for '{pattern}'");
                    }
                }
                Err(e) => eprintln!("codebook: invalid pattern '{pattern}': {e}"),
            }
        }
    }
    paths.sort();
    paths.dedup();
    paths
}

fn collect_dir(dir: &Path, out: &mut Vec<PathBuf>) {
    walkdir::WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .flatten()
        .filter(|e| e.file_type().is_file())
        .for_each(|e| out.push(e.into_path()));
}
