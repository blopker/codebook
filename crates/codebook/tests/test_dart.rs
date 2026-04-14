use codebook::queries::LanguageType;

mod utils;

#[test]
fn test_dart_simple() {
    utils::init_logging();
    let processor = utils::get_processor();
    let sample_text = r#"
import 'dart:math';
import 'package:flutter/materail.dart';

/// A documntation comment
class MyWidgett extends StatelessWidget {
  final String titlee;
  final int countr;

  const MyWidgett({required this.titlee, required this.countr});

  // Regular coment with misspeling
  String getMessge() {
    var greting = "Helllo Worlld";
    if (countr == 0) {
      return "No itemms";
    }
    return greting + " numbr $countr";
  }

  /* Block coment
   * with misspeled words
   */
  List<String> getItemms() {
    return ["firstt", "seconnd", "thirdd"];
  }
}

enum Statuss {
  activve,
  inacive,
}
    "#;
    let expected = vec![
        "Helllo",
        "Itemms",
        "Messge",
        "Statuss",
        "Widgett",
        "Worlld",
        "activve",
        "coment",
        "countr",
        "documntation",
        "firstt",
        "greting",
        "inacive",
        "itemms",
        "misspeled",
        "misspeling",
        "numbr",
        "seconnd",
        "thirdd",
    ];
    let not_expected = vec![
        "dart",     // import URI content
        "flutter",  // import URI content
        "materail", // import URI content (misspelled but in import)
        "package",  // import URI content
    ];
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Dart), None)
        .to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);
    for word in &not_expected {
        assert!(
            !misspelled.contains(word),
            "'{word}' should not be spell-checked (import URI)"
        );
    }
}
