#[test]
fn test_suggestions() {
    let processor = super::utils::get_processor();
    let suggestions = processor.get_suggestions("testz");
    assert!(!suggestions.unwrap().is_empty());
}
