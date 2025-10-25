use codebook::{
    parser::{TextRange, WordLocation},
    queries::LanguageType,
};

mod utils;

#[test]
fn test_typst_locations() {
    let sample_text = r#"
+ The climat
  - Temperature
  - Precipitation
+ The topography
+ The gology

= Introduction
In this report, we will explore the
various factors that influence _fluiid_ ..

= Glaciers

Glaciers as the one shown in
@glacierss will cease to exist if
we don't take action soon!

#figurr(
  imaget("glacierr-pic.jpg", width: 70%),
  caption: [
    _Glaciers_ form an important part
    of the earth's climate system.
  ],
) <glacierss>

The flow rate of a glacier is given
by the following equation:

$ Q = rho A v + "time offse" $

= Methods
We follow the glacier melting models
established in @glacier-meltt.

#biblography("works.bib")
    "#;

    let expected = vec![
        WordLocation::new(
            "climat".to_string(),
            vec![TextRange {
                start_byte: 7,
                end_byte: 13,
            }],
        ),
        WordLocation::new(
            "gology".to_string(),
            vec![TextRange {
                start_byte: 71,
                end_byte: 77,
            }],
        ),
        WordLocation::new(
            "fluiid".to_string(),
            vec![TextRange {
                start_byte: 162,
                end_byte: 168,
            }],
        ),
        WordLocation::new(
            "glacierr".to_string(),
            vec![TextRange {
                start_byte: 296,
                end_byte: 304,
            }],
        ),
        WordLocation::new(
            "glacierss".to_string(),
            vec![TextRange {
                start_byte: 422,
                end_byte: 431,
            }],
        ),
        WordLocation::new(
            "offse".to_string(),
            vec![TextRange {
                start_byte: 520,
                end_byte: 525,
            }],
        ),
    ];
    let not_expected = vec!["biblography", "figurr", "imaget", "meltt"];

    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Typst), None)
        .to_vec();
    println!("Misspelled words: {misspelled:?}");
    for e in &expected {
        println!("Expecting: {e:?}");
        let miss = misspelled
            .iter()
            .find(|r| r.word == e.word)
            .unwrap_or_else(|| panic!("Word '{}' not found in misspelled list", e.word));
        assert_eq!(miss.locations, e.locations);
    }
    for word in not_expected {
        assert!(!misspelled.iter().any(|r| r.word == word));
    }
}

#[test]
fn test_typst_imports() {
    let sample_text = r#"
// We ignore misspellings here
#import "conf.typ": confrnce
#import "conf.typ" as confernce
#import "label.typ": xlabl, ylabl, labl as lq-labbel
#import "@prevew/lable:1.1.1" as e
    "#;
    let expected = vec!["confernce", "labbel"];
    test_helper(sample_text, expected);
}

#[test]
fn test_typst_functions() {
    let sample_text = r#"
#show "once?": itly => [#itly #itly]

#let alrt(bdy, fil: red) = {
    set text(white)
    set align(center)
    // We ignore misspellings in the field names
    // we did not declare
    rect(
        fill: fil,
        inst: 8pt,
        radus: 4pt,
        [*Wrning:\ #bdy*],
    )
}

#alrt[
    Danger is imminent!
]

#alrt(fill: blue)[
    KEEP OFF TRAKS
]

#show heading.where(
    level: 1
): headng => block(width: 100%)[
    #set align(center)
    #set text(13pt, weight: "regular")
    #smallcaps(headng.body)
]

#let tmplate(docment) = [
    #set text(font: "Serif")
    #show "something cool": [Tywriter]
    #docment
]

#let left = (2, 4, 5)
#let right = (3, 2, 6)
#left.zip(right).map(
    ((lft, rght)) => lft + rght
)
    "#;
    let expected = vec![
        "TRAKS", "Tywriter", "Wrning", "alrt", "bdy", "docment", "fil", "headng", "itly", "lft",
        "rght", "tmplate",
    ];
    test_helper(sample_text, expected);
}

#[test]
fn test_typst_equations() {
    let sample_text = r#"
$ A = pi r^2 $
$ "area" = pi dot "radiu"^2 $
$ cal(A) :=
    { x in RR | x "is natral" } $
#let x = 5
$ #x < 17 $
    "#;
    let expected = vec!["natral", "radiu"];
    test_helper(sample_text, expected);
}

#[test]
fn test_typst_variable_declaration() {
    let sample_text = r#"
#let namee = "User"
This is #namee's docmentation.
Itt explains #namee.

#let add-variabls(aples, orng, ornges: string, aple: string) = aples + ornges
Sum is #add-variabls(2, 3).

#let (x, y) = (1, 2)
The coordinates are #x, #y.

#let (frst, .., lst) = (1, 2, 3, 4)
The first element is #frst.
The last element is #lst.

#let boks = (
    Shkespeare: "Hamlet",
    Homr: "The Odyssey",
    Austen: "Persuason",
)

#let (Austen,) = boks
Austen wrote #Austen.

#let (Homr: homerr, Austen: asten) = boks
Homer wrote #homerr.

#let (Austen, ..othr) = boks
#for (authr, ttle) in othr [
    #author wrote #title.
]
    "#;
    let expected = vec![
        "Homr",
        "Itt",
        "Persuason",
        "Shkespeare",
        "aple",
        "aples",
        "asten",
        "authr",
        "boks",
        "docmentation",
        "frst",
        "homerr",
        "lst",
        "namee",
        "orng",
        "ornges",
        "othr",
        "ttle",
        "variabls",
    ];
    test_helper(sample_text, expected);
}

#[test]
fn test_typst_comments() {
    let sample_text = r#"
// our data barely supprts
// this claim
/* Somebody wrte this up:
   - 1000 partcipants.
   - 2x2 data design. */
    "#;
    let expected = vec!["partcipants", "supprts", "wrte"];
    test_helper(sample_text, expected);
}

fn test_helper(sample_text: &str, expected: Vec<&str>) {
    utils::init_logging();
    let processor = utils::get_processor();
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Typst), None)
        .to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);
}
