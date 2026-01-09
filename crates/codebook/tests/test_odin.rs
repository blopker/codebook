use codebook::{
    parser::{TextRange, WordLocation},
    queries::LanguageType,
};

mod utils;

#[test]
fn test_odin_location() {
    utils::init_logging();
    let sample_text = include_str!("./examples/example.odin");
    let expected = vec![
        // Comments
        WordLocation::new(
            "Commennt".to_string(),
            vec![TextRange { start_byte: 196, end_byte: 204 }],
        ),
        WordLocation::new(
            "cooment".to_string(),
            vec![TextRange { start_byte: 215, end_byte: 222 }],
        ),
        WordLocation::new(
            "Netsed".to_string(),
            vec![TextRange { start_byte: 229, end_byte: 235 }],
        ),
        // Procedure declarations
        WordLocation::new(
            "proecdure".to_string(),
            vec![TextRange { start_byte: 253, end_byte: 262 }],
        ),
        WordLocation::new(
            "porocedure".to_string(),
            vec![TextRange { start_byte: 379, end_byte: 389 }],
        ),
        WordLocation::new(
            "overloded".to_string(),
            vec![TextRange { start_byte: 498, end_byte: 507 }],
        ),
        WordLocation::new(
            "deafult".to_string(),
            vec![TextRange { start_byte: 565, end_byte: 572 }],
        ),
        WordLocation::new(
            "varidic".to_string(),
            vec![TextRange { start_byte: 635, end_byte: 642 }],
        ),
        // Parameters
        WordLocation::new(
            "prameter".to_string(),
            vec![
                TextRange { start_byte: 274, end_byte: 282 },
                TextRange { start_byte: 401, end_byte: 409 },
                TextRange { start_byte: 584, end_byte: 592 },
            ],
        ),
        WordLocation::new(
            "paramter".to_string(),
            vec![
                TextRange { start_byte: 297, end_byte: 305 },
                TextRange { start_byte: 424, end_byte: 432 },
            ],
        ),
        WordLocation::new(
            "numberes".to_string(),
            vec![TextRange { start_byte: 651, end_byte: 659 }],
        ),
        // Constants
        WordLocation::new(
            "CONSATANT".to_string(),
            vec![TextRange { start_byte: 699, end_byte: 708 }],
        ),
        WordLocation::new(
            "COONSTANT".to_string(),
            vec![TextRange { start_byte: 729, end_byte: 738 }],
        ),
        WordLocation::new(
            "TWOOF".to_string(),
            vec![TextRange { start_byte: 1448, end_byte: 1453 }],
        ),
        // Variable declarations
        WordLocation::new(
            "assignement".to_string(),
            vec![
                TextRange { start_byte: 783, end_byte: 794 },
                TextRange { start_byte: 845, end_byte: 856 },
                TextRange { start_byte: 864, end_byte: 875 },
            ],
        ),
        WordLocation::new(
            "anotther".to_string(),
            vec![TextRange { start_byte: 811, end_byte: 819 }],
        ),
        WordLocation::new(
            "annother".to_string(),
            vec![TextRange { start_byte: 825, end_byte: 833 }],
        ),
        // Strings
        WordLocation::new(
            "Helloep".to_string(),
            vec![TextRange { start_byte: 937, end_byte: 944 }],
        ),
        WordLocation::new(
            "Wordl".to_string(),
            vec![TextRange { start_byte: 948, end_byte: 953 }],
        ),
        WordLocation::new(
            "Helolo".to_string(),
            vec![TextRange { start_byte: 1969, end_byte: 1975 }],
        ),
        WordLocation::new(
            "Wlorld".to_string(),
            vec![TextRange { start_byte: 1977, end_byte: 1983 }],
        ),
        // Struct declarations and fields
        WordLocation::new(
            "Awseome".to_string(),
            vec![TextRange { start_byte: 1153, end_byte: 1160 }],
        ),
        WordLocation::new(
            "Compacot".to_string(),
            vec![TextRange { start_byte: 1299, end_byte: 1307 }],
        ),
        WordLocation::new(
            "aples".to_string(),
            vec![TextRange { start_byte: 1328, end_byte: 1333 }],
        ),
        WordLocation::new(
            "banananas".to_string(),
            vec![TextRange { start_byte: 1335, end_byte: 1344 }],
        ),
        WordLocation::new(
            "ornages".to_string(),
            vec![TextRange { start_byte: 1346, end_byte: 1353 }],
        ),
        // Enum declaration and members
        WordLocation::new(
            "Cratfy".to_string(),
            vec![TextRange { start_byte: 1462, end_byte: 1468 }],
        ),
        WordLocation::new(
            "Aapple".to_string(),
            vec![TextRange { start_byte: 1485, end_byte: 1491 }],
        ),
        WordLocation::new(
            "Baanana".to_string(),
            vec![TextRange { start_byte: 1495, end_byte: 1502 }],
        ),
        WordLocation::new(
            "Oranege".to_string(),
            vec![TextRange { start_byte: 1510, end_byte: 1517 }],
        ),
        // Union declaration
        WordLocation::new(
            "Unberakable".to_string(),
            vec![TextRange { start_byte: 1583, end_byte: 1594 }],
        ),
        // Bit field declaration and members
        WordLocation::new(
            "Frutty".to_string(),
            vec![TextRange { start_byte: 1625, end_byte: 1631 }],
        ),
        WordLocation::new(
            "verison".to_string(),
            vec![TextRange { start_byte: 1664, end_byte: 1671 }],
        ),
        WordLocation::new(
            "ttl".to_string(),
            vec![TextRange { start_byte: 1691, end_byte: 1694 }],
        ),
        WordLocation::new(
            "opration".to_string(),
            vec![TextRange { start_byte: 1750, end_byte: 1758 }],
        ),
        WordLocation::new(
            "opernd".to_string(),
            vec![TextRange { start_byte: 1782, end_byte: 1788 }],
        ),
        WordLocation::new(
            "oprand".to_string(),
            vec![TextRange { start_byte: 1811, end_byte: 1817 }],
        ),
    ];
    let not_expected = ["fmt", "println"];
    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Odin), None)
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
