use codebook::queries::LanguageType;

use super::utils::assert_spelling_at;

/// The goal is to spellcheck comments, strings, and identifiers during
/// declarations, but not during usage — repeated words in the sample are
/// pinned to their declaration occurrence with occurrence indices.
#[test]
fn test_odin_location() {
    let sample_text = include_str!("../examples/example.odin");
    assert_spelling_at(
        LanguageType::Odin,
        sample_text,
        &[
            // Comments
            ("Commennt", &[0]),
            ("cooment", &[0]),
            ("Netsed", &[0]),
            // Procedure declarations
            ("proecdure", &[0]),
            ("porocedure", &[0]),
            ("overloded", &[0]),
            ("deafult", &[0]),
            ("varidic", &[0]),
            // Parameters (flagged in each proc's parameter list, not at usages)
            ("prameter", &[0, 2, 4]),
            ("paramter", &[0, 2]),
            ("numberes", &[0]),
            // Constants
            ("CONSATANT", &[0]),
            ("COONSTANT", &[0]),
            ("TWOOF", &[0]),
            // Variable declarations
            ("assignement", &[0, 1, 2]),
            ("anotther", &[0]),
            ("annother", &[0]),
            // Strings
            ("Helloep", &[0]),
            ("Wordl", &[0]),
            ("Helolo", &[0]),
            ("Wlorld", &[0]),
            // Struct declarations and fields
            ("Awseome", &[0]),
            ("Compacot", &[0]),
            ("aples", &[0]),
            ("banananas", &[0]),
            ("ornages", &[0]),
            // Enum declaration and members
            ("Cratfy", &[0]),
            ("Aapple", &[0]),
            ("Baanana", &[0]),
            ("Oranege", &[0]),
            // Union declaration
            ("Unberakable", &[0]),
            // Bit field declaration and members
            ("Frutty", &[0]),
            ("verison", &[0]),
            ("opration", &[0]),
            ("opernd", &[0]),
            ("oprand", &[0]),
        ],
    );
}
