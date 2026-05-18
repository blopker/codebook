use codebook::queries::LanguageType;

#[test]
fn test_dart_simple() {
    super::utils::init_logging();
    let processor = super::utils::get_processor();
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
        "titlee",
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

#[test]
fn test_dart_string_interpolation() {
    super::utils::init_logging();
    let processor = super::utils::get_processor();
    let sample_text = r#"
class Foo {
  String usrname = "bob";

  void doStuff() {
    var greting = "Helllo";
    logg('Deleeted accaunt for $usrname');
    print("Greting is $greting and numbr ${usrname.length}");
  }
}
    "#;
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Dart), None)
        .to_vec();
    let mut misspelled: Vec<&str> = binding.iter().map(|r| r.word.as_str()).collect();
    misspelled.sort();
    println!("Misspelled words: {misspelled:?}");

    let expected = vec![
        "Deleeted",
        "Greting",
        "Helllo",
        "accaunt",
        "greting",
        "numbr",
        "usrname",
    ];
    let not_expected = vec![
        "bob",    // string content, valid word
        "logg",   // function call (reference, not definition)
        "length", // interpolation expression member
    ];
    assert_eq!(misspelled, expected);
    for word in &not_expected {
        assert!(
            !misspelled.contains(word),
            "'{word}' should not be spell-checked (interpolation)"
        );
    }
}

#[test]
fn test_dart_class_fields() {
    super::utils::init_logging();
    let processor = super::utils::get_processor();
    let sample_text = r#"
class UserAccaunt {
  final String usrname;
  final Map<String, UserAccaunt> _accaunts = {};
  int _balanse = 0;
}
    "#;
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Dart), None)
        .to_vec();
    let mut misspelled: Vec<&str> = binding.iter().map(|r| r.word.as_str()).collect();
    misspelled.sort();
    println!("Misspelled words: {misspelled:?}");

    // Class fields should be flagged at definitions
    assert!(misspelled.contains(&"usrname"), "class field 'usrname' should be flagged");
    assert!(misspelled.contains(&"accaunts"), "class field '_accaunts' should be flagged");
    assert!(misspelled.contains(&"balanse"), "class field '_balanse' should be flagged");
    assert!(misspelled.contains(&"Accaunt"), "class name 'UserAccaunt' should be flagged at definition");

    // Type references should NOT be flagged
    let type_ref_words = vec!["String", "Map", "int"];
    for word in &type_ref_words {
        assert!(
            !misspelled.contains(word),
            "type reference '{word}' should not be spell-checked"
        );
    }
}

#[test]
fn test_dart_arrow_body_strings() {
    super::utils::init_logging();
    let processor = super::utils::get_processor();
    let sample_text = r#"
class Foo {
  String usrname = "bob";
  int _balanse = 0;

  @override
  String toString() => 'Accaunt(usrname: $usrname, balanse: $_balanse)';
}
    "#;
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Dart), None)
        .to_vec();
    let mut misspelled: Vec<&str> = binding.iter().map(|r| r.word.as_str()).collect();
    misspelled.sort();
    println!("Misspelled words: {misspelled:?}");

    // Arrow body string content should be checked
    assert!(misspelled.contains(&"Accaunt"), "string in arrow body should be checked");
    assert!(misspelled.contains(&"balanse"), "field definition should be flagged");
    assert!(misspelled.contains(&"usrname"), "field definition should be flagged");
}

#[test]
fn test_dart_type_references_not_flagged() {
    super::utils::init_logging();
    let processor = super::utils::get_processor();
    let sample_text = r#"
typedef MyCallbak = void Function(int);

mixin Loggabel {}

class Foo with Loggabel {
  final Map<String, List<int>> dataa = {};
  final MyCallbak? callbak;

  Foo({required this.dataa, this.callbak});

  Future<List<String>> fetchStuf() async {
    return [];
  }
}
    "#;
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Dart), None)
        .to_vec();
    let mut misspelled: Vec<&str> = binding.iter().map(|r| r.word.as_str()).collect();
    misspelled.sort();
    println!("Misspelled words: {misspelled:?}");

    // Definitions should be flagged
    assert!(misspelled.contains(&"Callbak"), "typedef name should be flagged");
    assert!(misspelled.contains(&"Loggabel"), "mixin name should be flagged");
    assert!(misspelled.contains(&"dataa"), "field name should be flagged");
    assert!(misspelled.contains(&"callbak"), "field name should be flagged");
    assert!(misspelled.contains(&"Stuf"), "method name should be flagged");

    // Type references should NOT be flagged
    let not_expected = vec![
        "Map", "String", "List", "int", "Future", "Loggabel", "MyCallbak",
    ];
    for word in &not_expected {
        // Skip words that are also flagged as definitions
        if *word == "Loggabel" || *word == "MyCallbak" {
            continue;
        }
        assert!(
            !misspelled.contains(word),
            "type reference '{word}' should not be spell-checked"
        );
    }
}
