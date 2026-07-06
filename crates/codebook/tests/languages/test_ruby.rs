use codebook::queries::LanguageType;

use super::utils::assert_spelling;

#[test]
fn test_ruby_simple() {
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
    assert_spelling(
        LanguageType::Ruby,
        sample_text,
        &[
            "comented", "helo", "inented", "lne", "lteral", "opttions", "sepaate", "wors",
        ],
        &[],
    );
}

#[test]
fn test_ruby_heredoc() {
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
    assert_spelling(
        LanguageType::Ruby,
        sample_ruby_heredocs,
        &[
            "Clickk",
            "documantation",
            "profesional",
            "importent",
            "Wellcome",
        ],
        &[],
    );
}

#[test]
fn test_ruby_code() {
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
    // Method call names ("mark_as_finnished") are not checked.
    assert_spelling(
        LanguageType::Ruby,
        sample_ruby_code,
        &[
            "potentialy",
            "compleet",
            "notfication",
            "Regads",
            "complette",
            "custommer",
            "Suport",
            "partialy",
        ],
        &["finnished"],
    );
}
