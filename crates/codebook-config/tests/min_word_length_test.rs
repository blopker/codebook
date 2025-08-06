use codebook_config::CodebookConfig;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_min_word_length_from_config() {
    // Create a temporary directory for the test
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("codebook.toml");

    // Write a config file with custom min_word_length
    let config_content = r#"
        dictionaries = ["en_us"]
        min_word_length = 2
    "#;
    fs::write(&config_path, config_content).unwrap();

    // Load the configuration
    let config = CodebookConfig::load(Some(temp_dir.path())).unwrap();

    // Verify the min_word_length is set correctly
    assert_eq!(config.get_min_word_length(), 2);
}

#[test]
fn test_min_word_length_default() {
    // Create a temporary directory for the test
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("codebook.toml");

    // Write a config file without min_word_length
    let config_content = r#"
        dictionaries = ["en_us"]
    "#;
    fs::write(&config_path, config_content).unwrap();

    // Load the configuration
    let config = CodebookConfig::load(Some(temp_dir.path())).unwrap();

    // Verify the default min_word_length is 3
    assert_eq!(config.get_min_word_length(), 3);
}

#[test]
fn test_min_word_length_with_global_config() {
    // Create temporary directories for global and project configs
    let global_dir = TempDir::new().unwrap();
    let project_dir = TempDir::new().unwrap();

    // Set up global config directory
    let global_config_dir = global_dir.path().join("config");
    fs::create_dir_all(&global_config_dir).unwrap();
    let global_config_path = global_config_dir.join("codebook.toml");

    // Write global config with min_word_length = 4
    let global_config_content = r#"
        min_word_length = 4
    "#;
    fs::write(&global_config_path, global_config_content).unwrap();

    // Write project config with min_word_length = 2
    let project_config_path = project_dir.path().join("codebook.toml");
    let project_config_content = r#"
        min_word_length = 2
    "#;
    fs::write(&project_config_path, project_config_content).unwrap();

    // Mock the XDG_CONFIG_HOME environment variable for this test
    unsafe {
        std::env::set_var("XDG_CONFIG_HOME", global_dir.path());
    }

    // Load the configuration
    let config = CodebookConfig::load(Some(project_dir.path())).unwrap();

    // Project config should override global config
    assert_eq!(config.get_min_word_length(), 2);

    // Clean up environment variable
    unsafe {
        std::env::remove_var("XDG_CONFIG_HOME");
    }
}

#[test]
fn test_min_word_length_zero() {
    // Create a temporary directory for the test
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("codebook.toml");

    // Write a config file with min_word_length = 0 (check all words)
    let config_content = r#"
        min_word_length = 0
    "#;
    fs::write(&config_path, config_content).unwrap();

    // Load the configuration
    let config = CodebookConfig::load(Some(temp_dir.path())).unwrap();

    // Verify min_word_length can be set to 0
    assert_eq!(config.get_min_word_length(), 0);
}

#[test]
fn test_min_word_length_large_value() {
    // Create a temporary directory for the test
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("codebook.toml");

    // Write a config file with a large min_word_length
    let config_content = r#"
        min_word_length = 10
    "#;
    fs::write(&config_path, config_content).unwrap();

    // Load the configuration
    let config = CodebookConfig::load(Some(temp_dir.path())).unwrap();

    // Verify large values work correctly
    assert_eq!(config.get_min_word_length(), 10);
}
