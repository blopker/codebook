use std::sync::LazyLock;

use super::dictionary::{Dictionary, HunspellDictionary};

static CODEBOOK_DICTIONARY: &str = include_str!("../../../../word_lists/combined.gen.txt");

#[derive(Clone, Debug)]
pub struct HunspellRepo {
    pub aff_url: String,
    pub dict_url: String,
    pub name: String,
}

impl HunspellRepo {
    pub fn new(name: &str, aff_url: &str, dict_url: &str) -> Self {
        Self {
            aff_url: aff_url.to_string(),
            dict_url: dict_url.to_string(),
            name: name.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TextRepo {
    pub url: String,
    pub name: String,
}

impl TextRepo {
    pub fn new(name: &str, url: &str) -> Self {
        Self {
            url: url.to_string(),
            name: name.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum DictionaryRepo {
    Hunspell(HunspellRepo),
    Text(TextRepo),
}

static DICTIONARIES: LazyLock<Vec<DictionaryRepo>> = LazyLock::new(|| {
    vec![DictionaryRepo::Hunspell(
    HunspellRepo::new(
        "en_us",
        "https://raw.githubusercontent.com/streetsidesoftware/cspell-dicts/refs/heads/main/dictionaries/en_US/src/hunspell/en_US-large.aff",
        "https://raw.githubusercontent.com/streetsidesoftware/cspell-dicts/refs/heads/main/dictionaries/en_US/src/hunspell/en_US-large.dic",
    )),
    DictionaryRepo::Hunspell(
    HunspellRepo::new(
        "en",
        "https://raw.githubusercontent.com/blopker/dictionaries/refs/heads/main/dictionaries/en/index.aff",
        "https://raw.githubusercontent.com/blopker/dictionaries/refs/heads/main/dictionaries/en/index.dic",
    )),
    DictionaryRepo::Hunspell(
    HunspellRepo::new(
        "en_gb",
        "https://raw.githubusercontent.com/streetsidesoftware/cspell-dicts/refs/heads/main/dictionaries/en_GB/src/hunspell/en_GB-large.aff",
        "https://raw.githubusercontent.com/streetsidesoftware/cspell-dicts/refs/heads/main/dictionaries/en_GB/src/hunspell/en_GB-large.dic",
    )),
    DictionaryRepo::Text(
    TextRepo::new(
        "rust",
        "https://raw.githubusercontent.com/streetsidesoftware/cspell-dicts/refs/heads/main/dictionaries/rust/dict/rust.txt",
    )),
    ]
});

pub fn get_codebook_dictionary() -> impl Iterator<Item = &'static str> {
    CODEBOOK_DICTIONARY.lines().filter(|l| !l.contains('#'))
}

pub fn get_repo(name: &str) -> Option<DictionaryRepo> {
    let res = DICTIONARIES.iter().find(|d| match d {
        DictionaryRepo::Hunspell(h) => h.name == name,
        DictionaryRepo::Text(t) => t.name == name,
    });

    match res {
        Some(d) => Some(d.clone()),
        None => None,
    }
}
