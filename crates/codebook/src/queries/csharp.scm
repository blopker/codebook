; Comments
(comment) @comment

;; Methods / functions
(method_declaration
  name: (identifier) @identifier.function)

(local_function_statement
  name: (identifier) @identifier.function)

(constructor_declaration
  name: (identifier) @identifier.function)

(destructor_declaration
  name: (identifier) @identifier.function)

;; Parameters
(parameter
  name: (identifier) @identifier.parameter)

; Variable/Field definitions
; local variables
(local_declaration_statement
  (variable_declaration
    (variable_declarator
      (identifier) @identifier.variable)))

; fields in classes/structs
(field_declaration
  (variable_declaration
    (variable_declarator
      (identifier) @identifier.field)))

; Struct/Type definitions
(interface_declaration
  name: (identifier) @identifier.type)

(class_declaration
  name: (identifier) @identifier.type)

(enum_declaration
  name: (identifier) @identifier.type)

(struct_declaration
  (identifier) @identifier.type)

(record_declaration
  (identifier) @identifier.type)

(namespace_declaration
  name: (identifier) @identifier.module)

(enum_member_declaration
  (identifier) @identifier.constant)

; String literals
(interpolated_string_expression
    (string_content) @string)

[
  (string_literal)
  (raw_string_literal)
  (verbatim_string_literal)
] @string
