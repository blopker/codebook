use codebook::Codebook;
use codebook_config::{CodebookConfig, CodebookConfigFile};
use std::collections::HashSet;
use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

struct Styles {
    bold: &'static str,
    dim: &'static str,
    yellow: &'static str,
    bold_red: &'static str,
    reset: &'static str,
}

const STYLES_ON: Styles = Styles {
    bold: "\x1b[1m",
    dim: "\x1b[2m",
    yellow: "\x1b[33m",
    bold_red: "\x1b[1;31m",
    reset: "\x1b[0m",
};

const STYLES_OFF: Styles = Styles {
    bold: "",
    dim: "",
    yellow: "",
    bold_red: "",
    reset: "",
};

fn styles(is_terminal: bool) -> &'static Styles {
    if is_terminal && std::env::var_os("NO_COLOR").is_none() {
        &STYLES_ON
    } else {
        &STYLES_OFF
    }
}

fn fatal(msg: impl std::fmt::Display, s: &Styles) -> ! {
    eprintln!("{}error:{} {msg}", s.bold_red, s.reset);
    std::process::exit(2);
}

pub fn run_lint(files: &[String], root: &Path, unique: bool) -> bool {
    let out = styles(std::io::stdout().is_terminal());
    let err = styles(std::io::stderr().is_terminal());

    let config = Arc::new(
        CodebookConfigFile::load(Some(root))
            .unwrap_or_else(|e| fatal(format!("failed to load config: {e}"), err)),
    );

    print_config_source(&config, err);

    let codebook = Codebook::new(config.clone())
        .unwrap_or_else(|e| fatal(format!("failed to initialize: {e}"), err));

    let resolved = resolve_paths(files);
    if resolved.is_empty() {
        fatal("no files matched the given patterns", err);
    }

    let mut seen_words: HashSet<String> = HashSet::new();
    let mut total_errors = 0usize;
    let mut files_with_errors = 0usize;

    for path in &resolved {
        if config.should_ignore_path(path) {
            continue;
        }

        let text = match std::fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!(
                    "{}error:{} {}: {e}",
                    err.bold_red,
                    err.reset,
                    path.display()
                );
                continue;
            }
        };

        let mut locations = codebook.spell_check(&text, None, path.to_str());
        locations.sort_by_key(|l| l.locations.first().map(|r| r.start_byte).unwrap_or(0));

        let hits: Vec<(usize, usize, &str)> = locations
            .iter()
            .flat_map(|wl| wl.locations.iter().map(move |r| (wl, r)))
            .filter_map(|(wl, range)| {
                if unique && !seen_words.insert(wl.word.to_lowercase()) {
                    return None;
                }
                let (line, col) = byte_offset_to_line_col(&text, range.start_byte);
                Some((line, col, wl.word.as_str()))
            })
            .collect();

        if hits.is_empty() {
            continue;
        }

        let raw = path.to_string_lossy();
        let display = raw.strip_prefix("./").unwrap_or(&raw);
        let pad_len = hits
            .iter()
            .map(|(l, c, _)| format!("{l}:{c}").len())
            .max()
            .unwrap_or(0);

        println!("{}{display}{}", out.bold, out.reset);
        for (line, col, word) in &hits {
            let loc = format!("{line}:{col}");
            println!(
                "  {}{display}{}:{}{loc}{}{pad}  {}{}{}",
                out.dim,
                out.reset,
                out.yellow,
                out.reset,
                out.bold_red,
                word,
                out.reset,
                pad = " ".repeat(pad_len - loc.len()),
            );
        }
        println!();

        total_errors += hits.len();
        files_with_errors += 1;
    }

    if total_errors > 0 {
        let _ = std::io::stdout().flush();
        let unique_label = if unique { "unique " } else { "" };
        eprintln!(
            "Found {}{total_errors}{} {unique_label}spelling error(s) in {}{files_with_errors}{} file(s).",
            err.bold, err.reset, err.bold, err.reset,
        );
    }

    total_errors > 0
}

fn print_config_source(config: &CodebookConfigFile, s: &Styles) {
    let cwd = std::env::current_dir().unwrap_or_default();
    match (
        config.project_config_path().filter(|p| p.is_file()),
        config.global_config_path().filter(|p| p.is_file()),
    ) {
        (Some(p), _) => eprintln!(
            "using config {}{}{}",
            s.dim,
            p.strip_prefix(&cwd).unwrap_or(&p).display(),
            s.reset
        ),
        (None, Some(g)) => eprintln!(
            "using global config {}{}{}",
            s.dim,
            g.strip_prefix(&cwd).unwrap_or(&g).display(),
            s.reset
        ),
        (None, None) => eprintln!("using default config"),
    }
}

fn resolve_paths(patterns: &[String]) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for pattern in patterns {
        let p = PathBuf::from(pattern);
        if p.is_dir() {
            collect_dir(&p, &mut paths);
        } else {
            match glob::glob(pattern) {
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

fn byte_offset_to_line_col(text: &str, byte_offset: usize) -> (usize, usize) {
    let offset = byte_offset.min(text.len());
    let before = &text[..offset];
    let line = before.bytes().filter(|&b| b == b'\n').count() + 1;
    let col = before.rfind('\n').map(|p| offset - p).unwrap_or(offset + 1);
    (line, col)
}
