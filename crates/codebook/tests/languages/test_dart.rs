use codebook::queries::LanguageType;

use super::utils::assert_spelling_at;

#[test]
fn test_dart_simple() {
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
    // Import URI contents (dart, package, flutter, and even the misspelled
    // materail) are not spell-checked; exact set equality guards that.
    assert_spelling_at(
        LanguageType::Dart,
        sample_text,
        &[
            ("documntation", &[0]),
            // Flagged at the class definition; the `const MyWidgett({...})`
            // constructor is not.
            ("Widgett", &[0]),
            // Fields are flagged at their definitions; `this.titlee`,
            // `this.countr`, and the `$countr` interpolation are not.
            ("titlee", &[0]),
            ("countr", &[0]),
            // Both the line comment and the block comment are checked.
            ("coment", &[0, 1]),
            ("misspeling", &[0]),
            ("misspeled", &[0]),
            ("Messge", &[0]),
            // Flagged at the declaration; the later usage is not.
            ("greting", &[0]),
            ("Helllo", &[0]),
            ("Worlld", &[0]),
            ("itemms", &[0]),
            ("numbr", &[0]),
            ("Itemms", &[0]),
            ("firstt", &[0]),
            ("seconnd", &[0]),
            ("thirdd", &[0]),
            ("Statuss", &[0]),
            ("activve", &[0]),
            ("inacive", &[0]),
        ],
    );
}

#[test]
fn test_dart_string_interpolation() {
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
    // Not flagged: "bob" (valid word in string content), `logg` (function
    // call reference, not a definition), and `length` (interpolation
    // expression member).
    assert_spelling_at(
        LanguageType::Dart,
        sample_text,
        &[
            // Flagged at the field definition; the `$usrname` and
            // `${usrname.length}` interpolation references are not.
            ("usrname", &[0]),
            // Flagged at the declaration; the `$greting` interpolation is not.
            ("greting", &[0]),
            ("Helllo", &[0]),
            // String content around interpolations is checked.
            ("Deleeted", &[0]),
            ("accaunt", &[0]),
            ("Greting", &[0]),
            ("numbr", &[0]),
        ],
    );
}

#[test]
fn test_dart_class_fields() {
    let sample_text = r#"
class UserAccaunt {
  final String usrname;
  final Map<String, UserAccaunt> _accaunts = {};
  int _balanse = 0;
}
    "#;
    // Type references (String, Map, int, and the `UserAccaunt` inside the
    // Map generic) are not spell-checked; exact set equality guards that.
    assert_spelling_at(
        LanguageType::Dart,
        sample_text,
        &[
            // Class name flagged at the definition only.
            ("Accaunt", &[0]),
            // Class fields are flagged at their definitions.
            ("usrname", &[0]),
            ("accaunts", &[0]),
            ("balanse", &[0]),
        ],
    );
}

#[test]
fn test_dart_arrow_body_strings() {
    let sample_text = r#"
class Foo {
  String usrname = "bob";
  int _balanse = 0;

  @override
  String toString() => 'Accaunt(usrname: $usrname, balanse: $_balanse)';
}
    "#;
    assert_spelling_at(
        LanguageType::Dart,
        sample_text,
        &[
            // String content in the arrow body is checked.
            ("Accaunt", &[0]),
            // Flagged at the field definition and at the literal text inside
            // the arrow-body string; the `$usrname` interpolation is not.
            ("usrname", &[0, 1]),
            ("balanse", &[0, 1]),
        ],
    );
}

#[test]
fn test_dart_type_references_not_flagged() {
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
    // Type references (Map, String, List, int, Future) are not
    // spell-checked; exact set equality guards that.
    assert_spelling_at(
        LanguageType::Dart,
        sample_text,
        &[
            // Flagged at the typedef definition; the `MyCallbak?` field type
            // reference is not.
            ("Callbak", &[0]),
            // Flagged at the mixin definition; the `with Loggabel` reference
            // is not.
            ("Loggabel", &[0]),
            // Fields are flagged at definitions; `this.dataa`/`this.callbak`
            // are not.
            ("dataa", &[0]),
            ("callbak", &[0]),
            ("Stuf", &[0]),
        ],
    );
}
