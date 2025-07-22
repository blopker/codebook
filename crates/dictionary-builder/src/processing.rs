//! Text processing utilities.

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use tracing::{debug, info};

use crate::Result;
use crate::error::Error;

/// Process multiple word list files into a single, sorted, deduplicated word list
pub fn process_word_list(sources: &[impl AsRef<Path>], output: &Path) -> Result<()> {
    let mut words = HashSet::new();

    // Process each source file
    for source in sources {
        let source_path = source.as_ref();
        debug!("Processing word list from {}", source_path.display());

        let file = File::open(source_path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Split on whitespace and non-word characters
            let parts: Vec<&str> = trimmed.split(|c: char| !c.is_alphanumeric()).collect();
            for part in parts {
                let word = part.trim();
                if !word.is_empty() {
                    words.insert(word.to_string());
                }
            }
        }
    }

    // Sort the words
    let mut sorted_words: Vec<_> = words.into_iter().collect();
    sorted_words.sort();

    // Write to output file
    let mut output_file = File::create(output)?;
    for word in sorted_words.iter() {
        writeln!(output_file, "{word}")?;
    }

    info!(
        "Processed {} source files into {} words at {}",
        sources.len(),
        sorted_words.len(),
        output.display()
    );

    Ok(())
}

/// Validate a Hunspell dictionary
pub fn validate_hunspell_dictionary(dic_path: &Path, aff_path: &Path) -> Result<()> {
    // Basic validation - just check if the files exist and are readable
    if !dic_path.exists() {
        return Err(Error::Processing(format!(
            "Dictionary file not found: {}",
            dic_path.display()
        )));
    }

    if !aff_path.exists() {
        return Err(Error::Processing(format!(
            "Affix file not found: {}",
            aff_path.display()
        )));
    }

    // Try to read the first line of the .dic file to check if it's valid
    let dic_file = File::open(dic_path)?;
    let mut dic_reader = BufReader::new(dic_file);
    let mut first_line = String::new();
    dic_reader.read_line(&mut first_line)?;

    // The first line should be a number (word count)
    let _word_count = first_line.trim().parse::<usize>().map_err(|_| {
        Error::Processing(format!(
            "Invalid Hunspell dictionary format at {}",
            dic_path.display()
        ))
    })?;

    debug!("Validated Hunspell dictionary: {}", dic_path.display());
    Ok(())
}
