use std::sync::Arc;

use codebook::Codebook;

pub fn get_processor() -> Codebook {
    let config = Arc::new(codebook_config::CodebookConfig::default());
    let dict = Codebook::new(config).unwrap();
    dict
}

pub fn init_logging() {
    let _ = env_logger::builder().is_test(true).try_init();
}
