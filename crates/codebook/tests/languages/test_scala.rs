use codebook::queries::LanguageType;

use super::utils::{assert_spelling, assert_spelling_at};


#[test]
fn test_scala_simple() {
    let sample_text = r#"
        package my.packkkkge
        
        class Calculatr(prrty: Boolean) {
          // This is an exampl module that performz calculashuns
          def add(numbr1: Int, numbr2: Int): String = {
            val resalt = numbr1 + numbr2
            if (prrty) s"*** ${numbr1 + numbr2} ***"  else (numbr1 + numbr2).toString
          }
        }
    "#;
    // "prtty" is flagged only on class parameters definition, usage is not flagged
    // "numbr" is flagged at the two parameter definitions only; the usages
    // in `numbr1 + numbr2` are not definitions. "resalt" is flagged at the
    // `=` binding, not at the bare return usage.
    assert_spelling_at(
        LanguageType::Scala,
        sample_text,
        &[
            ("packkkkge", &[0]),
            ("Calculatr", &[0]),
            ("prrty", &[0]),
            ("exampl", &[0]),
            ("performz", &[0]),
            ("calculashuns", &[0]),
            ("numbr", &[0, 1]),
            ("resalt", &[0]),
        ],
    );
}

#[test]
fn test_scala_type_parameters() {
    let sample_text = r#"
        trait Cnvertr[F[_], In, Ouput] {
            def convert(in: In): F[Ouput]
        }
    "#;
    // "Ouput" is flagged only on trait type parameters definition,
    // usage in the method return type is not flagged
    // One-letter type parameter "F" is not flagged
    assert_spelling_at(
        LanguageType::Scala,
        sample_text,
        &[
            ("Cnvertr", &[0]),
            ("Ouput", &[0]),
        ],
    );
}

#[test]
fn test_scala_pattern_matching() {
    let sample_text = r#"
        val (lft, zight) = (1, 2)
        List(lft, zight).map { case intgr =>
            intgr + 1
        } match {
            case haad :: secnd :: thrrrrd :: Nil => haad :: thrrrrd :: Nil
            case bindddd => binddd
        }
    "#;
    // All pattern bindings are flagged only on definition
    assert_spelling_at(
        LanguageType::Scala,
        sample_text,
        &[
            ("lft", &[0]),
            ("zight", &[0]),
            ("intgr", &[0]),
            ("haad", &[0]),
            ("secnd", &[0]),
            ("thrrrrd", &[0]),
            ("bindddd", &[0]),
        ],
    );
}

#[test]
fn test_scala_type_alias() {
    let sample_text= r#"
        type Nymbrr = Int
    "#;
    assert_spelling(LanguageType::Scala, sample_text, &["Nymbrr"], &[]);
}
