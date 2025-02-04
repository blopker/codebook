use std::sync::LazyLock;

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
    pub url: Option<String>,
    pub text: Option<&'static str>,
    pub name: String,
}

impl TextRepo {
    pub fn new(name: &str, url: &str) -> Self {
        Self {
            url: Some(url.to_string()),
            text: None,
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
    DictionaryRepo::Text(
    TextRepo{
        name: "codebook".to_string(),
        text: Some(CODEBOOK_DICTIONARY),
        url: None
    }),
    ]
});

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
