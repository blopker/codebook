use log::LevelFilter;
use serde::Deserialize;
use serde::de::Deserializer;
use serde_json::Value;
use std::path::PathBuf;

fn default_log_level() -> LevelFilter {
    LevelFilter::Info
}

fn deserialize_log_level<'de, D>(deserializer: D) -> Result<LevelFilter, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s.as_deref() {
        Some("trace") => Ok(LevelFilter::Trace),
        Some("debug") => Ok(LevelFilter::Debug),
        Some("warn") => Ok(LevelFilter::Warn),
        Some("error") => Ok(LevelFilter::Error),
        _ => Ok(LevelFilter::Info),
    }
}

fn deserialize_global_config_path<'de, D>(deserializer: D) -> Result<Option<PathBuf>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(path) => {
            let trimmed = path.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                Ok(Some(PathBuf::from(trimmed)))
            }
        }
        None => Ok(None),
    }
}

fn default_check_while_typing() -> bool {
    true
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ClientInitializationOptions {
    #[serde(
        default = "default_log_level",
        deserialize_with = "deserialize_log_level"
    )]
    pub(crate) log_level: LevelFilter,
    #[serde(default, deserialize_with = "deserialize_global_config_path")]
    pub(crate) global_config_path: Option<PathBuf>,
    #[serde(default = "default_check_while_typing")]
    pub(crate) check_while_typing: bool,
}

impl Default for ClientInitializationOptions {
    fn default() -> Self {
        ClientInitializationOptions {
            log_level: default_log_level(),
            global_config_path: None,
            check_while_typing: true,
        }
    }
}

impl ClientInitializationOptions {
    pub(crate) fn from_value(options_value: Option<Value>) -> Self {
        match options_value {
            None => ClientInitializationOptions::default(),
            Some(value) => match serde_json::from_value(value) {
                Ok(options) => options,
                Err(err) => {
                    log::error!(
                        "Failed to deserialize client initialization options. Using default: {}",
                        err
                    );
                    ClientInitializationOptions::default()
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_default() {
        let default_options = ClientInitializationOptions::default();
        assert_eq!(default_options.log_level, LevelFilter::Info);
        assert!(default_options.check_while_typing);
    }

    #[test]
    fn test_custom() {
        let custom_options = ClientInitializationOptions {
            log_level: LevelFilter::Debug,
            check_while_typing: false,
            ..Default::default()
        };
        assert_eq!(custom_options.log_level, LevelFilter::Debug);
        assert!(!custom_options.check_while_typing);
    }

    #[test]
    fn test_json() {
        let json = r#"{"logLevel": "debug", "checkWhileTyping": false}"#;
        let options: ClientInitializationOptions = serde_json::from_str(json).unwrap();
        assert_eq!(options.log_level, LevelFilter::Debug);
        assert!(!options.check_while_typing);
    }
}
