use codebook::downloader::DictionaryDownloader;

#[test]
fn test_downloader() {
    let downloader = DictionaryDownloader::with_cache("../.cache/dictionaries");
    let files = downloader.get("en").unwrap();
    assert_eq!(files.aff_local_path, "../.cache/dictionaries/en_index.aff");
    assert_eq!(files.dic_local_path, "../.cache/dictionaries/en_index.dic");
}
