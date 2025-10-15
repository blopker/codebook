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
                start_byte: 13,
                end_byte: 23,
            }],
        ),
        WordLocation::new(
            "alicz".to_string(),
            vec![TextRange {
                start_byte: 427,
                end_byte: 432,
            }],
        ),
        WordLocation::new(
            "myfmt".to_string(),
            vec![TextRange {
                start_byte: 35,
                end_byte: 40,
            }],
        ),
        WordLocation::new(
            "Userr".to_string(),
            vec![
                TextRange {
                    start_byte: 56,
                    end_byte: 61,
                },
                TextRange {
                    start_byte: 158,
                    end_byte: 163,
                },
                TextRange {
                    start_byte: 175,
                    end_byte: 180,
                },
                TextRange {
                    start_byte: 229,
                    end_byte: 234,
                },
                TextRange {
                    start_byte: 239,
                    end_byte: 244,
                },
                TextRange {
                    start_byte: 261,
                    end_byte: 266,
                },
                TextRange {
                    start_byte: 284,
                    end_byte: 289,
                },
            ],
        ),
        WordLocation::new(
            "prefixx".to_string(),
            vec![TextRange {
                start_byte: 245,
                end_byte: 252,
            }],
        ),
        WordLocation::new(
            "Namee".to_string(),
            vec![
                TextRange {
                    start_byte: 79,
                    end_byte: 84,
                },
                TextRange {
                    start_byte: 200,
                    end_byte: 205,
                },
            ],
        ),
        WordLocation::new(
            "Servicce".to_string(),
            vec![TextRange {
                start_byte: 126,
                end_byte: 134,
            }],
        ),
        WordLocation::new(
            "namme".to_string(),
            vec![TextRange {
                start_byte: 99,
                end_byte: 104,
            }],
        ),
        WordLocation::new(
            "outerr".to_string(),
            vec![
                TextRange {
                    start_byte: 634,
                    end_byte: 640,
                },
                TextRange {
                    start_byte: 738,
                    end_byte: 744,
                },
            ],
        ),
        WordLocation::new(
            "itemns".to_string(),
            vec![TextRange {
                start_byte: 777,
                end_byte: 783,
            }],
        ),
        WordLocation::new(
            "indexx".to_string(),
            vec![TextRange {
                start_byte: 838,
                end_byte: 844,
            }],
        ),
        WordLocation::new(
            "cokbookkk".to_string(),
            vec![TextRange {
                start_byte: 559,
                end_byte: 568,
            }],
        ),
        WordLocation::new(
            "valie".to_string(),
            vec![TextRange {
                start_byte: 578,
                end_byte: 583,
            }],
        ),
        WordLocation::new(
            "Wolrd".to_string(),
            vec![TextRange {
                start_byte: 406,
                end_byte: 411,
            }],
        ),
        WordLocation::new(
            "Alicz".to_string(),
            vec![TextRange {
                start_byte: 436,
                end_byte: 441,
            }],
        ),
        WordLocation::new(
            "speling".to_string(),
            vec![TextRange {
                start_byte: 362,
                end_byte: 369,
            }],
        ),
        WordLocation::new(
            "Hellol".to_string(),
            vec![TextRange {
                start_byte: 466,
                end_byte: 472,
            }],
        ),
        WordLocation::new(
            "imdex".to_string(),
            vec![TextRange {
                start_byte: 654,
                end_byte: 659,
            }],
        ),
        WordLocation::new(
            "firstt".to_string(),
            vec![TextRange {
                start_byte: 797,
                end_byte: 803,
            }],
        ),
        WordLocation::new(
            "seconnd".to_string(),
            vec![TextRange {
                start_byte: 807,
                end_byte: 814,
            }],
        ),
        WordLocation::new(
            "tihrd".to_string(),
            vec![TextRange {
                start_byte: 818,
                end_byte: 823,
            }],
        ),
        WordLocation::new(
            "valuue".to_string(),
            vec![TextRange {
                start_byte: 846,
                end_byte: 852,
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
        // assert_eq!(miss.locations, e.locations);
        assert!(miss.locations.len() == e.locations.len());
        for location in &miss.locations {
            assert!(e.locations.contains(location));
        }
    }
    for result in misspelled {
        assert!(!not_expected.contains(&result.word.as_str()));
    }
}
