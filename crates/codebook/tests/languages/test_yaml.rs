use codebook::queries::LanguageType;

use super::utils::{assert_spelling, assert_spelling_at};

#[test]
fn test_yaml_simple() {
    let sample_text = r#"
      # On a sepaate line
      title: "Example lne"
      nested:
        name: Naame
        nested:
          item_name: Iteem name
      details: |
        # can be inented
        This is comented out
        this wors, too
      options:
        - opttions
        - parameters
      flags: froozen
      var: 'helo'
      symbol: ':hello'
    "#;
    assert_spelling(
        LanguageType::YAML,
        sample_text,
        &[
            "Iteem", "Naame", "comented", "froozen", "helo", "inented", "lne", "opttions",
            "sepaate", "wors",
        ],
        &[],
    );
}

#[test]
fn test_yaml_code() {
    let sample_yaml_code = r#"
      # On a separate line
      tiitle: "Example line"
      subtiitle: Subtitle
      descriptioon: 'hello'
      nested_struucture:
        name: "Name"
        nestted:
          item_name: 'Item name'
          another_name: Another Name
      options:
        - parameters:
        - parameters
      items: [ { id: 1, naame: "one" }, { id: 2, name: "two" } ]

    "#;
    // "tiitle" also occurs inside "subtiitle" (occurrence 1); only the
    // standalone key is flagged as "tiitle" — the "subtiitle" key is flagged
    // as its own word.
    assert_spelling_at(
        LanguageType::YAML,
        sample_yaml_code,
        &[
            ("tiitle", &[0]),
            ("subtiitle", &[0]),
            ("descriptioon", &[0]),
            ("struucture", &[0]),
            ("nestted", &[0]),
            ("naame", &[0]),
        ],
    );
}

#[test]
fn test_yaml_ci_scripts_are_checked_as_shell() {
    // CI `run:`/`script:` values are shell scripts, checked with the Bash
    // grammar: command names and flags aren't words, but the script's
    // comments and strings are. Dependabot ecosystem names are vocabulary.
    let sample_yaml_code = r#"
steps:
  - run: |
      # Prepair the build
      mkdir -p "$RUNNER_TEMP/codebook"
      chmod +x ./script.sh && sed -i 's/a/b/' file.txt
      myprojtool --frobnicate; journalctl -u sshd | tee out.log
      echo "Deploying the aplication"
  - run: kubectl apply -f deploy.yaml
build:
  script:
    - npx xyzzytool --quux
  before_script:
    - podman pull thingy
updates:
  - package-ecosystem: gitsubmodule
    directory: "/"
    schedule:
      interval: weekly
  - package-ecosystem: npm
    note: a wrongg value
"#;
    assert_spelling_at(
        LanguageType::YAML,
        sample_yaml_code,
        &[("Prepair", &[0]), ("aplication", &[0]), ("wrongg", &[0])],
    );
}

#[test]
fn test_yaml_escape_sequences_separate_words() {
    let sample_text = r#"key: "this\nis\tindented wrold""#;
    assert_spelling_at(LanguageType::YAML, sample_text, &[("wrold", &[0])]);
}
