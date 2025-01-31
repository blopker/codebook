use codebook_config::CodebookConfig;

fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = CodebookConfig::load()?;

    // Use the configuration
    println!(
        "Loaded dictionaries: {:?}",
        config.get_settings().dictionaries
    );

    // Check if a path should be ignored
    let should_ignore = config.should_ignore_path("target/debug/build");
    println!("Should ignore path: {}", should_ignore);

    // Check if a word is allowed
    let is_allowed = config.is_allowed_word("rustc");
    println!("Is 'rustc' allowed: {}", is_allowed);

    // Check if a word should be flagged
    let should_flag = config.should_flag_word("todo");
    println!("Should flag 'todo': {}", should_flag);

    Ok(())
}
