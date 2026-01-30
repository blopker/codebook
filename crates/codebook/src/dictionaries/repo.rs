use std::sync::LazyLock;

static CODEBOOK_DICTIONARY: &str = include_str!("./combined.gen.txt");

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
            "https://raw.githubusercontent.com/blopker/dictionaries/refs/heads/main/dictionaries/en-GB/index.aff",
            "https://raw.githubusercontent.com/blopker/dictionaries/refs/heads/main/dictionaries/en-GB/index.dic",
        ),
        HunspellRepo::new(
            "es",
            "https://raw.githubusercontent.com/blopker/dictionaries/refs/heads/main/dictionaries/es/index.aff",
            "https://raw.githubusercontent.com/blopker/dictionaries/refs/heads/main/dictionaries/es/index.dic",
        ),
        HunspellRepo::new(
            "de",
            "https://raw.githubusercontent.com/blopker/dictionaries/refs/heads/main/dictionaries/de/index.aff",
            "https://raw.githubusercontent.com/blopker/dictionaries/refs/heads/main/dictionaries/de/index.dic",
        ),
        HunspellRepo::new(
            "de_at",
            "https://raw.githubusercontent.com/blopker/dictionaries/refs/heads/main/dictionaries/de-AT/index.aff",
            "https://raw.githubusercontent.com/blopker/dictionaries/refs/heads/main/dictionaries/de-AT/index.dic",
        ),
        HunspellRepo::new(
            "de_ch",
            "https://raw.githubusercontent.com/blopker/dictionaries/refs/heads/main/dictionaries/de-CH/index.aff",
            "https://raw.githubusercontent.com/blopker/dictionaries/refs/heads/main/dictionaries/de-CH/index.dic",
        ),
        HunspellRepo::new(
            "fr",
            "https://raw.githubusercontent.com/blopker/dictionaries/refs/heads/main/dictionaries/fr/index.aff",
            "https://raw.githubusercontent.com/blopker/dictionaries/refs/heads/main/dictionaries/fr/index.dic",
        ),
        HunspellRepo::new(
            "ru",
            "https://raw.githubusercontent.com/blopker/dictionaries/refs/heads/main/dictionaries/ru/index.aff",
            "https://raw.githubusercontent.com/blopker/dictionaries/refs/heads/main/dictionaries/ru/index.dic",
        ),
        HunspellRepo::new(
            "it",
            "https://raw.githubusercontent.com/blopker/dictionaries/refs/heads/main/dictionaries/it/index.aff",
            "https://raw.githubusercontent.com/blopker/dictionaries/refs/heads/main/dictionaries/it/index.dic",
        ),
        HunspellRepo::new(
            "sv",
            "https://raw.githubusercontent.com/blopker/dictionaries/refs/heads/main/dictionaries/sv/index.aff",
            "https://raw.githubusercontent.com/blopker/dictionaries/refs/heads/main/dictionaries/sv/index.dic",
        ),
        HunspellRepo::new(
            "pt_br",
            "https://raw.githubusercontent.com/streetsidesoftware/cspell-dicts/refs/heads/main/dictionaries/pt_BR/src/hunspell/index.aff",
            "https://raw.githubusercontent.com/streetsidesoftware/cspell-dicts/refs/heads/main/dictionaries/pt_BR/src/hunspell/index.dic",
        ),
        HunspellRepo::new(
            "nl_nl",
            "https://raw.githubusercontent.com/blopker/dictionaries/refs/heads/main/dictionaries/nl/index.aff",
            "https://raw.githubusercontent.com/blopker/dictionaries/refs/heads/main/dictionaries/nl/index.dic",
        ),
        HunspellRepo::new(
            "da",
            "https://raw.githubusercontent.com/blopker/dictionaries/refs/heads/main/dictionaries/da/index.aff",
            "https://raw.githubusercontent.com/blopker/dictionaries/refs/heads/main/dictionaries/da/index.dic",
        ),
        HunspellRepo::new(
            "vi_vn",
            "https://raw.githubusercontent.com/streetsidesoftware/cspell-dicts/main/dictionaries/vi_VN/src/hunspell/vi.aff",
            "https://raw.githubusercontent.com/streetsidesoftware/cspell-dicts/main/dictionaries/vi_VN/src/hunspell/vi.dic",
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
        TextRepo::new(
            "csharp",
            "https://raw.githubusercontent.com/streetsidesoftware/cspell-dicts/refs/heads/main/dictionaries/csharp/csharp.txt",
        ),
    ]
});

pub fn get_repo(name: &str) -> Option<DictionaryRepo> {
    let res = HUNSPELL_DICTIONARIES.iter().find(|d| d.name == name);
    if let Some(res1) = res {
        return Some(DictionaryRepo::Hunspell(res1.clone()));
    }
    let res = TEXT_DICTIONARIES.iter().find(|d| d.name == name);
    if let Some(res1) = res {
        return Some(DictionaryRepo::Text(res1.clone()));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_dictionary_names_unique_and_snake_case() {
        // Test that dictionary names are unique across both collections
        let hunspell_names: HashSet<String> = HUNSPELL_DICTIONARIES
            .iter()
            .map(|d| d.name.clone())
            .collect();

        let text_names: HashSet<String> =
            TEXT_DICTIONARIES.iter().map(|d| d.name.clone()).collect();

        // Check for duplicates within each collection
        assert_eq!(
            hunspell_names.len(),
            HUNSPELL_DICTIONARIES.len(),
            "Duplicate names found in HUNSPELL_DICTIONARIES"
        );

        assert_eq!(
            text_names.len(),
            TEXT_DICTIONARIES.len(),
            "Duplicate names found in TEXT_DICTIONARIES"
        );

        // Check for overlaps between collections
        let mut all_names = HashSet::new();
        let mut duplicate_names = Vec::new();

        for name in hunspell_names.iter().chain(text_names.iter()) {
            if !all_names.insert(name) {
                duplicate_names.push(name.clone());
            }
        }

        assert!(
            duplicate_names.is_empty(),
            "Found duplicate names across collections: {duplicate_names:?}"
        );

        // Test that all names follow snake_case convention
        let non_snake_case = all_names
            .iter()
            .filter(|name| {
                !name
                    .chars()
                    .all(|c| c.is_lowercase() || c == '_' || c.is_ascii_digit())
                    || name.contains("__")
                    || name.starts_with('_')
                    || name.ends_with('_')
            })
            .collect::<Vec<_>>();

        assert!(
            non_snake_case.is_empty(),
            "Found names not in snake_case format: {non_snake_case:?}"
        );
    }
}
