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
pub fn init_logging() {
    let _ = env_logger::builder().is_test(true).try_init();
}

pub fn example_file_path(file: &str) -> String {
    // get root of the project through CARGO_MANIFEST_DIR
    format!("tests/examples/{file}")
}
