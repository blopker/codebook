; Comments
(comment) @comment

;; Methods / functions
(method_declaration
  name: (identifier) @identifier)

(local_function_statement
  name: (identifier) @identifier)

(constructor_declaration
  name: (identifier) @identifier)

(destructor_declaration
  name: (identifier) @identifier)

;; Parameters
(parameter
  name: (identifier) @identifier)

; Variable/Field definitions
; local variables
(local_declaration_statement
  (variable_declaration
    (variable_declarator
      (identifier) @identifier)))

; fields in classes/structs
(field_declaration
  (variable_declaration
    (variable_declarator
      (identifier) @identifier)))

; Struct/Type definitions
(interface_declaration
  name: (identifier) @identifier)

(class_declaration
  name: (identifier) @identifier)

(enum_declaration
  name: (identifier) @identifier)

(struct_declaration
  (identifier) @identifier)

(record_declaration
  (identifier) @identifier)

(namespace_declaration
  name: (identifier) @identifier)

(enum_member_declaration
  (identifier) @identifier)

; String literals
(interpolated_string_expression
    (string_content) @string)

[
  (string_literal)
  (raw_string_literal)
  (verbatim_string_literal)
] @string
