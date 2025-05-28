use codebook::Codebook;
use codebook::queries::LanguageType;
use std::env;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Arc::new(codebook_config::CodebookConfig::default());
    let processor = Codebook::new(config).unwrap();

    println!("My path is {:?}", args);

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
        println!("Misspelled words: {:?}", misspelled);
        return;
    }

    let path = Path::new(args[1].as_str());
    if !path.exists() {
        eprintln!("Can't find file {path:?}");
        return;
    }
    let results = processor.spell_check_file(path.to_str().unwrap());
    println!("Misspelled words: {:?}", results);
    println!("Done");
}

#[cfg(target_os = "windows")]
fn run_benchmark(processor: &Codebook) {
    println!("Not supported in windows");
}

#[cfg(not(target_os = "windows"))]
fn run_benchmark(processor: &Codebook) {
    println!("Running spell_check benchmark...");
    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(998)
        .blocklist(&["libc", "libgcc", "pthread", "vdso"])
        .build()
        .unwrap();
    // Define sample text for benchmark
    let sample_text = include_str!("../../../examples/wulf.txt");

    // Number of iterations for benchmark
    let iterations = 100;

    // Start timing
    let start = Instant::now();

    // Run spell_check multiple times
    for i in 1..=iterations {
        if i % 10 == 0 {
            println!("Iteration {}/{}", i, iterations);
        }
        let _misspelled = processor.spell_check(sample_text, Some(LanguageType::Text), None);
    }

    // Calculate duration
    let duration = start.elapsed();
    let avg_time = duration.as_secs_f64() / iterations as f64;

    println!("\nBenchmark Results:");
    println!("Total iterations: {}", iterations);
    println!("Total time: {:.2?}", duration);
    println!(
        "Average time per iteration: {:.6?}",
        Duration::from_secs_f64(avg_time)
    );
    if let Ok(report) = guard.report().build() {
        let file = File::create("flamegraph.svg").unwrap();
        report.flamegraph(file).unwrap();
    };
}
