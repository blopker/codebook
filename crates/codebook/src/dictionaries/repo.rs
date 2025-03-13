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

static HUNSPELL_DICTIONARIES: LazyLock<Vec<HunspellRepo>> = LazyLock::new(|| {
    vec![
        HunspellRepo::new(
            "en_us",
            "https://raw.githubusercontent.com/streetsidesoftware/cspell-dicts/refs/heads/main/dictionaries/en_US/src/hunspell/en_US-large.aff",
            "https://raw.githubusercontent.com/streetsidesoftware/cspell-dicts/refs/heads/main/dictionaries/en_US/src/hunspell/en_US-large.dic",
        ),
        HunspellRepo::new(
            "en",
            "https://raw.githubusercontent.com/blopker/dictionaries/refs/heads/main/dictionaries/en/index.aff",
            "https://raw.githubusercontent.com/blopker/dictionaries/refs/heads/main/dictionaries/en/index.dic",
        ),
        HunspellRepo::new(
            "en_gb",
            "https://raw.githubusercontent.com/wooorm/dictionaries/refs/heads/main/dictionaries/en-GB/index.aff",
            "https://raw.githubusercontent.com/wooorm/dictionaries/refs/heads/main/dictionaries/en-GB/index.dic",
        ),
        HunspellRepo::new(
            "es",
            "https://raw.githubusercontent.com/wooorm/dictionaries/refs/heads/main/dictionaries/es/index.aff",
            "https://raw.githubusercontent.com/wooorm/dictionaries/refs/heads/main/dictionaries/es/index.dic",
        ),
        HunspellRepo::new(
            "de",
            "https://raw.githubusercontent.com/wooorm/dictionaries/refs/heads/main/dictionaries/de/index.aff",
            "https://raw.githubusercontent.com/wooorm/dictionaries/refs/heads/main/dictionaries/de/index.dic",
        ),
        HunspellRepo::new(
            "de-AT",
            "https://raw.githubusercontent.com/wooorm/dictionaries/refs/heads/main/dictionaries/de-AT/index.aff",
            "https://raw.githubusercontent.com/wooorm/dictionaries/refs/heads/main/dictionaries/de-AT/index.dic",
        ),
        HunspellRepo::new(
            "de-CH",
            "https://raw.githubusercontent.com/wooorm/dictionaries/refs/heads/main/dictionaries/de-CH/index.aff",
            "https://raw.githubusercontent.com/wooorm/dictionaries/refs/heads/main/dictionaries/de-CH/index.dic",
        ),
    ]
});

static TEXT_DICTIONARIES: LazyLock<Vec<TextRepo>> = LazyLock::new(|| {
    vec![
        TextRepo::new(
            "rust",
            "https://raw.githubusercontent.com/streetsidesoftware/cspell-dicts/refs/heads/main/dictionaries/rust/dict/rust.txt",
        ),
        TextRepo::new(
            "software_terms",
            "https://raw.githubusercontent.com/streetsidesoftware/cspell-dicts/refs/heads/main/dictionaries/software-terms/dict/softwareTerms.txt",
        ),
        TextRepo::new(
            "computing_acronyms",
            "https://raw.githubusercontent.com/streetsidesoftware/cspell-dicts/refs/heads/main/dictionaries/software-terms/dict/computing-acronyms.txt",
        ),
        TextRepo {
            name: "codebook".to_string(),
            text: Some(CODEBOOK_DICTIONARY),
            url: None,
        },
    ]
});

pub fn get_repo(name: &str) -> Option<DictionaryRepo> {
    let res = HUNSPELL_DICTIONARIES.iter().find(|d| d.name == name);
    if res.is_some() {
        return Some(DictionaryRepo::Hunspell(res.unwrap().clone()));
    }
    let res = TEXT_DICTIONARIES.iter().find(|d| d.name == name);
    if res.is_some() {
        return Some(DictionaryRepo::Text(res.unwrap().clone()));
    }
    None
}
