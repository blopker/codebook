use std::sync::LazyLock;

static CODEBOOK_DICTIONARY: &str = include_str!("../../../word_lists/combined.gen.txt");

#[derive(Clone, Debug)]
struct HunspellDictionaryLocation {
    pub aff_url: String,
    pub dict_url: String,
    pub name: String,
}

impl HunspellDictionaryLocation {
    pub fn new(name: &str, aff_url: &str, dict_url: &str) -> Self {
        Self {
            aff_url: aff_url.to_string(),
            dict_url: dict_url.to_string(),
            name: name.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
struct TextDictionaryLocation {
    pub url: String,
    pub name: String,
}

#[derive(Clone, Debug)]
enum DictionaryLocation {
    Hunspell(HunspellDictionaryLocation),
    Text(TextDictionaryLocation),
}

static NATRUAL_DICTIONARIES: LazyLock<Vec<DictionaryLocation>> = LazyLock::new(|| {
    vec![DictionaryLocation::Hunspell(
    HunspellDictionaryLocation::new(
        "en_us",
        "https://raw.githubusercontent.com/streetsidesoftware/cspell-dicts/refs/heads/main/dictionaries/en_US/src/hunspell/en_US-large.aff",
        "https://raw.githubusercontent.com/streetsidesoftware/cspell-dicts/refs/heads/main/dictionaries/en_US/src/hunspell/en_US-large.dic",
    )),
    DictionaryLocation::Hunspell(
    HunspellDictionaryLocation::new(
        "en_gb",
        "https://raw.githubusercontent.com/streetsidesoftware/cspell-dicts/refs/heads/main/dictionaries/en_GB/src/hunspell/en_GB-large.aff",
        "https://raw.githubusercontent.com/streetsidesoftware/cspell-dicts/refs/heads/main/dictionaries/en_GB/src/hunspell/en_GB-large.dic",
    )),
    ]
});

pub fn get_codebook_dictionary() -> impl Iterator<Item = &'static str> {
    CODEBOOK_DICTIONARY.lines().filter(|l| !l.contains('#'))
}

pub fn get_natural_dictionary(name: &str) -> Option<DictionaryLocation> {
    let res = NATRUAL_DICTIONARIES.iter().find(|d| match d {
        DictionaryLocation::Hunspell(h) => h.name == name,
        _ => false,
    });

    match res {
        Some(d) => Some(d.clone()),
        None => None,
    }
}
