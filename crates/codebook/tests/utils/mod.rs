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

#[allow(dead_code)]
pub fn get_processor_with_include(include: &str) -> Codebook {
    let config = Arc::new(CodebookConfigMemory::default());
    config
        .add_include(include)
        .expect("Should add include path");
    Codebook::new(config).unwrap()
}

#[allow(dead_code)]
pub fn get_processor_with_include_and_ignore(include: &str, ignore: &str) -> Codebook {
    let config = Arc::new(CodebookConfigMemory::default());
    config
        .add_include(include)
        .expect("Should add include path");
    config.add_ignore(ignore).expect("Should add ignore path");
    Codebook::new(config).unwrap()
}

#[allow(dead_code)]
pub fn init_logging() {
    let _ = env_logger::builder().is_test(true).try_init();
}
