use codebook::Codebook;
use codebook::queries::{LanguageType, get_language_name_from_filename, get_language_setting};
use std::env;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tree_sitter::Parser;

fn main() {
    let args: Vec<String> = env::args().collect();

    // --ast <file> [--lang <language>]: dump the tree-sitter AST for a file
    // Use --lang to override language detection, or use "-" for stdin.
    if args.contains(&"--ast".to_string()) {
        let lang_override = args
            .windows(2)
            .find(|w| w[0] == "--lang")
            .map(|w| w[1].as_str());
        let file_arg = args
            .iter()
            .find(|a| *a != "--ast" && *a != "--lang" && !a.starts_with("--") && !a.contains("codebook"));
        match file_arg {
            Some(path) => dump_ast(path, lang_override),
            None => eprintln!("Usage: codebook --ast <file> [--lang <language>]"),
        }
        return;
    }

    let config = Arc::new(codebook_config::CodebookConfigFile::load(None).unwrap());
    let processor = Codebook::new(config).unwrap();

    // Check for benchmark flag
    if args.contains(&"--benchmark".to_string()) {
        run_benchmark(&processor);
        return;
    }

    if args.len() < 2 {
        let sample_text = r#"
            fn calculate_user_age(bithDate: String) -> u32 {
                // This is a codebook example_function that calculates age
                let userAge = get_current_date() - birthDate;
                userAge
            }
        "#;

        let misspelled = processor.spell_check(sample_text, Some(LanguageType::Rust), None);
        println!("Misspelled words: {misspelled:?}");
        return;
    }

    let path = Path::new(args[1].as_str());
    if !path.exists() {
        eprintln!("Can't find file {path:?}");
        return;
    }
    let results = processor.spell_check_file(path.to_str().unwrap());
    println!("Misspelled words: {results:?}");
    println!("Done");
}

fn dump_ast(path: &str, lang_override: Option<&str>) {
    let lang_type = match lang_override {
        Some(lang) => lang.parse().unwrap_or(LanguageType::Text),
        None => get_language_name_from_filename(path),
    };
    let setting = match get_language_setting(lang_type) {
        Some(s) => s,
        None => {
            eprintln!("No tree-sitter grammar for {path} (detected as {lang_type:?})");
            return;
        }
    };
    let ts_lang = setting.language().unwrap();
    let source = if path == "-" {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf).unwrap();
        buf
    } else {
        std::fs::read_to_string(path).unwrap()
    };
    let mut parser = Parser::new();
    parser.set_language(&ts_lang).unwrap();
    let tree = parser.parse(&source, None).unwrap();

    println!("Language: {lang_type:?}");
    println!("---");
    print_node(tree.root_node(), &source, 0);
}

fn print_node(node: tree_sitter::Node, source: &str, indent: usize) {
    let text: String = node
        .utf8_text(source.as_bytes())
        .unwrap_or("")
        .chars()
        .take(60)
        .collect();
    println!(
        "{:indent$}{} [{}-{}] {text:?}",
        "",
        node.kind(),
        node.start_byte(),
        node.end_byte(),
        indent = indent
    );
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        print_node(child, source, indent + 2);
    }
}

#[cfg(target_os = "windows")]
fn run_benchmark(processor: &Codebook) {
    println!("Not supported in windows");
}

#[cfg(not(target_os = "windows"))]
fn run_benchmark(processor: &Codebook) {
    const ITERATIONS: usize = 100;

    // Run text benchmark
    benchmark_text_file(processor, ITERATIONS);

    // Run JavaScript benchmark
    benchmark_javascript_file(processor, ITERATIONS);
}

#[cfg(not(target_os = "windows"))]
fn benchmark_text_file(processor: &Codebook, iterations: usize) {
    println!("Running text file spell_check benchmark...");

    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(9998)
        .build()
        .unwrap();

    let sample_text = include_str!("../tests/examples/wulf.txt");

    let duration = run_benchmark_iterations(
        processor,
        sample_text,
        LanguageType::Text,
        iterations,
        "Text",
        Some(262), // Expected misspellings for assertion
    );

    print_benchmark_results("Text file", iterations, duration);

    // Generate flamegraph
    if let Ok(report) = guard.report().build() {
        let file = File::create("flamegraph_text.svg").unwrap();
        report.flamegraph(file).unwrap();
        println!("Text benchmark flamegraph saved to flamegraph_text.svg");
    }
}

#[cfg(not(target_os = "windows"))]
fn benchmark_javascript_file(processor: &Codebook, iterations: usize) {
    println!("\nRunning JavaScript code benchmark...");

    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(9998)
        .build()
        .unwrap();

    let js_code = include_str!("../tests/examples/example.js");

    let duration = run_benchmark_iterations(
        processor,
        js_code,
        LanguageType::Javascript,
        iterations,
        "JavaScript",
        Some(31),
    );

    print_benchmark_results("JavaScript file", iterations, duration);

    // Generate flamegraph
    if let Ok(report) = guard.report().build() {
        let file = File::create("flamegraph_js.svg").unwrap();
        report.flamegraph(file).unwrap();
        println!("JavaScript benchmark flamegraph saved to flamegraph_js.svg");
    }
}

#[cfg(not(target_os = "windows"))]
fn run_benchmark_iterations(
    processor: &Codebook,
    content: &str,
    language: LanguageType,
    iterations: usize,
    label: &str,
    expected_misspellings: Option<usize>,
) -> Duration {
    let start = Instant::now();

    for i in 1..=iterations {
        if i % 10 == 0 {
            println!("{label} iteration {i}/{iterations}");
        }
        let misspelled = processor.spell_check(content, Some(language), None);

        if let Some(expected) = expected_misspellings {
            assert_eq!(misspelled.len(), expected);
        }
    }

    start.elapsed()
}

#[cfg(not(target_os = "windows"))]
fn print_benchmark_results(name: &str, iterations: usize, duration: Duration) {
    let avg_time = duration.as_secs_f64() / iterations as f64;

    println!("\nBenchmark Results ({name}):");
    println!("Total iterations: {iterations}");
    println!("Total time: {duration:.2?}");
    println!(
        "Average time per iteration: {:.6?}",
        Duration::from_secs_f64(avg_time)
    );
}
