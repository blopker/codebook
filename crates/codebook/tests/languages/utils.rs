use std::sync::Arc;

use codebook::Codebook;
use codebook_config::{CodebookConfig, CodebookConfigMemory};

pub fn get_processor() -> Codebook {
    let config = Arc::new(CodebookConfigMemory::default());
    config
        .add_ignore("**/ignore.txt")
        .expect("Should ignore file");
    Codebook::new(config).unwrap()
}

pub fn get_processor_with_include(include: &str) -> Codebook {
    let config = Arc::new(CodebookConfigMemory::default());
    config
        .add_include(include)
        .expect("Should add include path");
    Codebook::new(config).unwrap()
}

pub fn get_processor_with_include_and_ignore(include: &str, ignore: &str) -> Codebook {
    let config = Arc::new(CodebookConfigMemory::default());
    config
        .add_include(include)
        .expect("Should add include path");
    config.add_ignore(ignore).expect("Should add ignore path");
    Codebook::new(config).unwrap()
}

pub fn get_processor_with_tags(include_tags: Vec<&str>, exclude_tags: Vec<&str>) -> Codebook {
    let settings = codebook_config::ConfigSettings {
        include_tags: include_tags.into_iter().map(String::from).collect(),
        exclude_tags: exclude_tags.into_iter().map(String::from).collect(),
        ..Default::default()
    };
    let config = Arc::new(CodebookConfigMemory::new(settings));
    Codebook::new(config).unwrap()
}

pub fn init_logging() {
    let _ = env_logger::builder().is_test(true).try_init();
}
