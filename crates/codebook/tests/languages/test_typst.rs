use codebook::queries::LanguageType;

use super::utils::{assert_spelling, assert_spelling_at};

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
    // Exact set equality: function names (figurr, imaget, biblography) and
    // reference usages (@glacier-meltt) can't appear in the flagged set.
    // "climat" also occurs inside "climate system" (occurrence 1);
    // "glacierss" is flagged at its label definition `<glacierss>`
    // (occurrence 1), not at the `@glacierss` usage.
    assert_spelling_at(
        LanguageType::Typst,
        sample_text,
        &[
            ("climat", &[0]),
            ("gology", &[0]),
            ("fluiid", &[0]),
            ("glacierr", &[0]),
            ("glacierss", &[1]),
            ("offse", &[0]),
        ],
    );
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
    // Imported names and import paths aren't checked; the aliases the user
    // picks (confernce, lq-labbel) are.
    assert_spelling(
        LanguageType::Typst,
        sample_text,
        &["confernce", "labbel"],
        &["confrnce", "xlabl", "ylabl", "prevew", "lable"],
    );
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
    // Identifiers are flagged at their definitions (parameters, closure
    // bindings), not at usages. "fil" occurrence 0 is the `fil: red`
    // parameter; its other matches are usages or substrings of "fill".
    assert_spelling_at(
        LanguageType::Typst,
        sample_text,
        &[
            ("TRAKS", &[0]),
            ("Tywriter", &[0]),
            ("Wrning", &[0]),
            ("alrt", &[0]),
            ("bdy", &[0]),
            ("docment", &[0]),
            ("fil", &[0]),
            ("headng", &[0]),
            ("itly", &[0]),
            ("lft", &[0]),
            ("rght", &[0]),
            ("tmplate", &[0]),
        ],
    );
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
    // Only quoted strings inside math are checked; math identifiers
    // (pi, dot, cal, RR) are not.
    assert_spelling(
        LanguageType::Typst,
        sample_text,
        &["natral", "radiu"],
        &["pi", "dot", "cal", "RR"],
    );
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
    // Identifiers are flagged at their definitions, not at usages: "Homr"
    // is flagged at the dictionary key, not at the destructuring pattern
    // that refers back to it. "aple" occurrence 1 is the standalone
    // parameter; occurrences 0 and 2 are substrings of "aples", which is
    // flagged as its own word.
    assert_spelling_at(
        LanguageType::Typst,
        sample_text,
        &[
            ("Homr", &[0]),
            ("Itt", &[0]),
            ("Persuason", &[0]),
            ("Shkespeare", &[0]),
            ("aple", &[1]),
            ("aples", &[0]),
            ("asten", &[0]),
            ("authr", &[0]),
            ("boks", &[0]),
            ("docmentation", &[0]),
            ("frst", &[0]),
            ("homerr", &[0]),
            ("lst", &[0]),
            ("namee", &[0]),
            ("orng", &[0]),
            ("ornges", &[0]),
            ("othr", &[0]),
            ("ttle", &[0]),
            ("variabls", &[0]),
        ],
    );
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
    assert_spelling(
        LanguageType::Typst,
        sample_text,
        &["partcipants", "supprts", "wrte"],
        &[],
    );
}
