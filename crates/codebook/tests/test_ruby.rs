use codebook::{
    parser::{TextRange, WordLocation},
    queries::LanguageType,
};

mod utils;

#[test]
fn test_ruby_simple() {
    utils::init_logging();
    let processor = utils::get_processor();
    let sample_text = r#"
        # On a sepaate line
        class Foo # or at the end of the lne
          # can be inented
          def bar
          end
          def opttions
          end
        end

        =begin
        This is
        comented out
        =end

        class Foo
        end

        =begin some_tag
        this wors, too
        =end

        # frozen_string_lteral: true

        var = 'helo'
        symbol = :hello
    "#;
    let expected = vec![
        "comented", "helo", "inented", "lne", "lteral", "opttions", "sepaate", "wors",
    ];
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Ruby), None)
        .to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);
}

#[test]
fn test_ruby_heredoc() {
    utils::init_logging();
    let sample_ruby_heredocs = r#"
instructions = %Q{
  1. Clickk on the "Forgot Password" link
}

long_text = <<~TEXT
  The documantation should be clear and profesional.
TEXT

sql_comment = <<~SQL
  -- It's importent to regularly clean up unverified accounts
SQL

html_content = <<-HTML
  <h1>Wellcome to our website!</h1>
HTML
        "#;

    let expected = vec![
        WordLocation::new(
            "Clickk".to_string(),
            vec![TextRange {
                start_byte: 5,
                end_byte: 11,
            }],
        ),
        WordLocation::new(
            "documantation".to_string(),
            vec![TextRange {
                start_byte: 6,
                end_byte: 19,
            }],
        ),
        WordLocation::new(
            "profesional".to_string(),
            vec![TextRange {
                start_byte: 40,
                end_byte: 51,
            }],
        ),
        WordLocation::new(
            "importent".to_string(),
            vec![TextRange {
                start_byte: 10,
                end_byte: 19,
            }],
        ),
        WordLocation::new(
            "Wellcome".to_string(),
            vec![TextRange {
                start_byte: 6,
                end_byte: 14,
            }],
        ),
    ];
    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_ruby_heredocs, Some(LanguageType::Ruby), None)
        .to_vec();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled.len(), expected.len());
    for e in &expected {
        let miss = misspelled.iter().find(|r| r.word == e.word).unwrap();
        println!("Expecting: {e:?}");
        assert_eq!(miss.locations, e.locations);
    }
}

#[test]
fn test_ruby_code() {
    utils::init_logging();
    let sample_ruby_code = r#"
def send_notfication(recipient, subject, body)
  # This method sends an email with potentialy misspelled content
  email = Email.new(
    to: recipient,
    subject: "URGENT: #{subject}",
    body: "Dear valued custommer,\n\n#{body}\n\nRegads,\nSuport Team"
  )
  email.send
end

if status == "complette" || status == "partialy_compleet"
  mark_as_finnished(item)
end
        "#;

    let expected = vec![
        WordLocation::new(
            "potentialy".to_string(),
            vec![TextRange {
                start_byte: 36,
                end_byte: 46,
            }],
        ),
        WordLocation::new(
            "compleet".to_string(),
            vec![TextRange {
                start_byte: 48,
                end_byte: 56,
            }],
        ),
        WordLocation::new(
            "notfication".to_string(),
            vec![TextRange {
                start_byte: 9,
                end_byte: 20,
            }],
        ),
        WordLocation::new(
            "Regads".to_string(),
            vec![TextRange {
                start_byte: 48,
                end_byte: 54,
            }],
        ),
        WordLocation::new(
            "complette".to_string(),
            vec![TextRange {
                start_byte: 14,
                end_byte: 23,
            }],
        ),
        WordLocation::new(
            "custommer".to_string(),
            vec![TextRange {
                start_byte: 23,
                end_byte: 32,
            }],
        ),
        WordLocation::new(
            "Suport".to_string(),
            vec![TextRange {
                start_byte: 57,
                end_byte: 63,
            }],
        ),
        WordLocation::new(
            "partialy".to_string(),
            vec![TextRange {
                start_byte: 39,
                end_byte: 47,
            }],
        ),
    ];
    let not_expected = vec!["finnished"];
    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_ruby_code, Some(LanguageType::Ruby), None)
        .to_vec();
    println!("Misspelled words: {misspelled:?}");
    for e in &expected {
        let miss = misspelled.iter().find(|r| r.word == e.word).unwrap();
        println!("Expecting: {e:?}");
        assert_eq!(miss.locations, e.locations);
    }
    for word in not_expected {
        println!("Not expecting: {word:?}");
        assert!(!misspelled.iter().any(|r| r.word == word));
    }
}
