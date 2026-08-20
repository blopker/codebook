use codebook::queries::LanguageType;

use super::utils::{assert_spelling, assert_spelling_at};

#[test]
fn test_hcl_attribute_identifiers() {
    let sample_text = r#"
resource "aws_instance" "web_server" {
    instnace_type = "t3.micro"
    descriptin    = "Valid text"
}
"#;

    assert_spelling(
        LanguageType::Hcl,
        sample_text,
        &["instnace", "descriptin"],
        &["resource", "aws_instance", "web_server"],
    );
}

#[test]
fn test_hcl_strings_and_comments() {
    let sample_text = r#"
# Comment with a speling error.

/* Multi-line comment
with an error on the seconnd line. */

variable "region" {
    description = "Cloud regon for the deploymant."
    default     = "eu-west-1"
}

resource "aws_instance" "web_server" {
    user_data = <<-EOT
        Configure the servver for this environment.
        Enable applicaton logging for ${var.region}.
    EOT
}
"#;

    assert_spelling_at(
        LanguageType::Hcl,
        sample_text,
        &[
            ("speling", &[0]),
            ("seconnd", &[0]),
            ("regon", &[0]),
            ("deploymant", &[0]),
            ("servver", &[0]),
            ("applicaton", &[0]),
        ],
    );
}

#[test]
fn test_hcl_only_checks_identifier_definitions() {
    let sample_text = r#"
variable "source_value" {
    default = "valid"
}

locals {
    misspeled_value = var.source_value
}

output "result" {
    value = local.misspeled_value
}
"#;

    assert_spelling_at(LanguageType::Hcl, sample_text, &[("misspeled", &[0])]);
}

#[test]
fn test_hcl_string_interpolation() {
    let sample_text = r#"
variable "region" {
    default = "eu-west-1"
}

locals {
    message = "Deploy to the regon ${var.misspeled_reference}"
}
"#;

    assert_spelling(
        LanguageType::Hcl,
        sample_text,
        &["regon"],
        &["misspeled_reference"],
    );
}
