use std::path::PathBuf;
use std::sync::Arc;

use codebook::Codebook;
use codebook_config::{CodebookConfig, CodebookConfigMemory};

/// Build a Codebook that loads dictionaries from the checked-in fixtures
/// instead of downloading — tests must not touch the network (the downloader
/// is compiled with deny-network in test builds and would panic). Refresh
/// fixtures with `make fetch_fixtures`.
pub fn make_codebook(config: Arc<dyn CodebookConfig>) -> Codebook {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/dictionaries");
    Codebook::with_dictionary_dir(config, Some(fixtures)).unwrap()
}

pub fn get_processor() -> Codebook {
    let config = Arc::new(CodebookConfigMemory::default());
    config
        .add_ignore("**/ignore.txt")
        .expect("Should ignore file");
    make_codebook(config)
}

pub fn get_processor_with_include(include: &str) -> Codebook {
    let config = Arc::new(CodebookConfigMemory::default());
    config
        .add_include(include)
        .expect("Should add include path");
    make_codebook(config)
}

pub fn get_processor_with_include_and_ignore(include: &str, ignore: &str) -> Codebook {
    let config = Arc::new(CodebookConfigMemory::default());
    config
        .add_include(include)
        .expect("Should add include path");
    config.add_ignore(ignore).expect("Should add ignore path");
    make_codebook(config)
}

pub fn get_processor_with_tags(include_tags: Vec<&str>, exclude_tags: Vec<&str>) -> Codebook {
    let settings = codebook_config::ConfigSettings {
        include_tags: include_tags.into_iter().map(String::from).collect(),
        exclude_tags: exclude_tags.into_iter().map(String::from).collect(),
        ..Default::default()
    };
    let config = Arc::new(CodebookConfigMemory::new(settings));
    make_codebook(config)
}

pub fn init_logging() {
    let _ = env_logger::builder().is_test(true).try_init();
}

/// Proves the deny-network dev-dependency feature reaches the downloader in
/// this crate's test builds: a dictionary miss (cold cache, no fixture) must
/// panic instead of downloading.
#[test]
#[should_panic(expected = "Blocked network request")]
fn network_guard_active_in_test_builds() {
    use codebook::dictionaries::manager::DictionaryManager;
    let temp_cache = tempfile::tempdir().unwrap();
    let manager = DictionaryManager::new(&temp_cache.path().to_path_buf());
    let _ = manager.get_dictionary("en_gb");
}
