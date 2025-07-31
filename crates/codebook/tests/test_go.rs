use codebook::{
    parser::{TextRange, WordLocation},
    queries::LanguageType,
};

mod utils;

#[test]
fn test_go_location() {
    utils::init_logging();
    let sample_text = r#"
    package pacagename
    import myfmt "fmt"
    type Userr struct {
        Namee string `json:"namme"`
    }
    type UserServicce interface {
        GetUserr(id string) Userr
    }
    const MaxNameeSize = 100
    func (u *Userr) GetUserr(prefixx string) Userr {
        return Userr{Namee: prefixx + "Alice"}
    }
    func main() {
        // I'm bad at speling alice
        myfmt.Println("Hello, Wolrd!")
        var alicz = "Alicz"
        myfmt.Println("Hellol, " + alicz)
        var rsvp = "RSVP"
        myfmt.Println("Hello, " + rsvp)
        cokbookkk := "test valie"
        myfmt.Println("Hello, " + cokbookkk)
    outerr:
        for imdex := 0; imdex < 10; imdex++ {
            if imdex == 5 {
                break outerr
            }
        }
        itemns := []string{"firstt", "seconnd", "tihrd"}
        for indexx, valuue := range itemns {
            myfmt.Println(indexx, valuue)
        }
        myfmt.Println(itemns)
    }"#;
    let expected = vec![
        WordLocation::new(
            "pacagename".to_string(),
            vec![TextRange {
                start_char: 12,
                end_char: 22,
                line: 1,
            }],
        ),
        WordLocation::new(
            "alicz".to_string(),
            vec![TextRange {
                start_char: 12,
                end_char: 17,
                line: 16,
            }],
        ),
        WordLocation::new(
            "myfmt".to_string(),
            vec![TextRange {
                start_char: 11,
                end_char: 16,
                line: 2,
            }],
        ),
        WordLocation::new(
            "Userr".to_string(),
            vec![TextRange {
                start_char: 9,
                end_char: 14,
                line: 3,
            }, TextRange {
                start_char: 11,
                end_char: 16,
                line: 7,
            }, TextRange {
                start_char: 28,
                end_char: 33,
                line: 7,
            }, TextRange {
                start_char: 13,
                end_char: 18,
                line: 10,
            }, TextRange {
                start_char: 23,
                end_char: 28,
                line: 10,
            }, TextRange {
                start_char: 45,
                end_char: 50,
                line: 10,
            }, TextRange {
                start_char: 15,
                end_char: 20,
                line: 11,
            }],
        ),
        WordLocation::new(
            "Namee".to_string(),
            vec![TextRange {
                start_char: 8,
                end_char: 13,
                line: 4,
            }, TextRange {
                start_char: 13,
                end_char: 18,
                line: 9,
            }],
        ),
        WordLocation::new(
            "Servicce".to_string(),
            vec![TextRange {
                start_char: 13,
                end_char: 21,
                line: 6,
            }],
        ),
        WordLocation::new(
            "namme".to_string(),
            vec![TextRange {
                start_char: 28,
                end_char: 33,
                line: 4,
            }],
        ),
        WordLocation::new(
            "outerr".to_string(),
            vec![TextRange {
                start_char: 4,
                end_char: 10,
                line: 22,
            }, TextRange {
                start_char: 22,
                end_char: 28,
                line: 25,
            }],
        ),
        WordLocation::new(
            "itemns".to_string(),
            vec![TextRange {
                start_char: 8,
                end_char: 14,
                line: 28,
            }],
        ),
        WordLocation::new(
            "indexx".to_string(),
            vec![TextRange {
                start_char: 12,
                end_char: 18,
                line: 29,
            }],
        ),
        WordLocation::new(
            "cokbookkk".to_string(),
            vec![TextRange {
                start_char: 8,
                end_char: 17,
                line: 20,
            }],
        ),
        WordLocation::new(
            "valie".to_string(),
            vec![TextRange {
                start_char: 27,
                end_char: 32,
                line: 20,
            }],
        ),
        WordLocation::new(
            "Wolrd".to_string(),
            vec![TextRange {
                start_char: 30,
                end_char: 35,
                line: 15,
            }],
        ),
        WordLocation::new(
            "Alicz".to_string(),
            vec![TextRange {
                start_char: 21,
                end_char: 26,
                line: 16,
            }],
        ),
        WordLocation::new(
            "speling".to_string(),
            vec![TextRange {
                start_char: 22,
                end_char: 29,
                line: 14,
            }],
        ),
        WordLocation::new(
            "Hellol".to_string(),
            vec![TextRange {
                start_char: 23,
                end_char: 29,
                line: 17,
            }],
        ),
        WordLocation::new(
            "imdex".to_string(),
            vec![TextRange {
                start_char: 12,
                end_char: 17,
                line: 23,
            }],
        ),
        WordLocation::new(
            "firstt".to_string(),
            vec![TextRange {
                start_char: 28,
                end_char: 34,
                line: 28,
            }],
        ),
        WordLocation::new(
            "seconnd".to_string(),
            vec![TextRange {
                start_char: 38,
                end_char: 45,
                line: 28,
            }],
        ),
        WordLocation::new(
            "tihrd".to_string(),
            vec![TextRange {
                start_char: 49,
                end_char: 54,
                line: 28,
            }],
        ),
        WordLocation::new(
            "valuue".to_string(),
            vec![TextRange {
                start_char: 20,
                end_char: 26,
                line: 29,
            }],
        ),
    ];
    let not_expected = ["fmt"];
    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Go), None)
        .to_vec();
    println!("Misspelled words: {misspelled:?}");
    for e in &expected {
        println!("Expecting: {e:?}");
        let miss = misspelled.iter().find(|r| r.word == e.word).unwrap();
        assert_eq!(miss.locations, e.locations);
    }
    for result in misspelled {
        assert!(!not_expected.contains(&result.word.as_str()));
    }
}
