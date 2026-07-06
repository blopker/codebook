use codebook::queries::LanguageType;

use super::utils::{assert_spelling, assert_spelling_at};

#[test]
fn test_vhdl_simple() {
    let sample_text = r#"
-- This is an exmple comment with speling errors
entity calculatr is
    port (
        clk     : in  std_logic;
        resett  : in  std_logic;
        inputt  : in  std_logic_vector(7 downto 0)
    );
end entity calculatr;
"#;
    assert_spelling_at(
        LanguageType::VHDL,
        sample_text,
        &[
            ("exmple", &[0]),
            ("speling", &[0]),
            // Entity names are flagged at declaration, not at the
            // `end entity` reference.
            ("calculatr", &[0]),
            ("clk", &[0]),
            ("resett", &[0]),
            ("inputt", &[0]),
        ],
    );
}

#[test]
fn test_vhdl_comment_location() {
    assert_spelling(
        LanguageType::VHDL,
        "\n-- A calculater for numbrs\n",
        &["calculater", "numbrs"],
        &[],
    );
}
