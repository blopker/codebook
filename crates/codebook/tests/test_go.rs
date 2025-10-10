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
                start_byte: 12,
                end_byte: 22,
            }],
        ),
        WordLocation::new(
            "alicz".to_string(),
            vec![TextRange {
                start_byte: 12,
                end_byte: 17,
            }],
        ),
        WordLocation::new(
            "myfmt".to_string(),
            vec![TextRange {
                start_byte: 11,
                end_byte: 16,
            }],
        ),
        WordLocation::new(
            "Userr".to_string(),
            vec![
                TextRange {
                    start_byte: 9,
                    end_byte: 14,
                },
                TextRange {
                    start_byte: 11,
                    end_byte: 16,
                },
                TextRange {
                    start_byte: 28,
                    end_byte: 33,
                },
                TextRange {
                    start_byte: 13,
                    end_byte: 18,
                },
                TextRange {
                    start_byte: 23,
                    end_byte: 28,
                },
                TextRange {
                    start_byte: 45,
                    end_byte: 50,
                },
                TextRange {
                    start_byte: 15,
                    end_byte: 20,
                },
            ],
        ),
        WordLocation::new(
            "Namee".to_string(),
            vec![
                TextRange {
                    start_byte: 8,
                    end_byte: 13,
                },
                TextRange {
                    start_byte: 13,
                    end_byte: 18,
                },
            ],
        ),
        WordLocation::new(
            "Servicce".to_string(),
            vec![TextRange {
                start_byte: 13,
                end_byte: 21,
            }],
        ),
        WordLocation::new(
            "namme".to_string(),
            vec![TextRange {
                start_byte: 28,
                end_byte: 33,
            }],
        ),
        WordLocation::new(
            "outerr".to_string(),
            vec![
                TextRange {
                    start_byte: 4,
                    end_byte: 10,
                },
                TextRange {
                    start_byte: 22,
                    end_byte: 28,
                },
            ],
        ),
        WordLocation::new(
            "itemns".to_string(),
            vec![TextRange {
                start_byte: 8,
                end_byte: 14,
            }],
        ),
        WordLocation::new(
            "indexx".to_string(),
            vec![TextRange {
                start_byte: 12,
                end_byte: 18,
            }],
        ),
        WordLocation::new(
            "cokbookkk".to_string(),
            vec![TextRange {
                start_byte: 8,
                end_byte: 17,
            }],
        ),
        WordLocation::new(
            "valie".to_string(),
            vec![TextRange {
                start_byte: 27,
                end_byte: 32,
            }],
        ),
        WordLocation::new(
            "Wolrd".to_string(),
            vec![TextRange {
                start_byte: 30,
                end_byte: 35,
            }],
        ),
        WordLocation::new(
            "Alicz".to_string(),
            vec![TextRange {
                start_byte: 21,
                end_byte: 26,
            }],
        ),
        WordLocation::new(
            "speling".to_string(),
            vec![TextRange {
                start_byte: 22,
                end_byte: 29,
            }],
        ),
        WordLocation::new(
            "Hellol".to_string(),
            vec![TextRange {
                start_byte: 23,
                end_byte: 29,
            }],
        ),
        WordLocation::new(
            "imdex".to_string(),
            vec![TextRange {
                start_byte: 12,
                end_byte: 17,
            }],
        ),
        WordLocation::new(
            "firstt".to_string(),
            vec![TextRange {
                start_byte: 28,
                end_byte: 34,
            }],
        ),
        WordLocation::new(
            "seconnd".to_string(),
            vec![TextRange {
                start_byte: 38,
                end_byte: 45,
            }],
        ),
        WordLocation::new(
            "tihrd".to_string(),
            vec![TextRange {
                start_byte: 49,
                end_byte: 54,
            }],
        ),
        WordLocation::new(
            "valuue".to_string(),
            vec![TextRange {
                start_byte: 20,
                end_byte: 26,
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
