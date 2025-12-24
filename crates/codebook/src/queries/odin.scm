; Comments
(comment) @comment
(block_comment) @comment

; Procedure declarations (including parameter names)
(procedure_declaration
  (expression) @identifier)
(overloaded_procedure_declaration
  (expression) @identifier)
(parameter
  (identifier) @identifier)
(default_parameter
  (identifier) @identifier)

; Variables and constants identifiers (declaration-only)
(var_declaration
  (expression) @identifier ":")
(assignment_statement
  (expression) @identifier ":=")
(const_declaration
  (expression)+ @identifier)
(const_type_declaration
  (expression)+ @identifier)

; Struct, enum, union, bit_fields names
(struct_declaration
  (expression) @identifier)
(enum_declaration
  (expression) @identifier)
(union_declaration
  (expression) @identifier)
(bit_field_declaration
  (expression) @identifier "::")

; Field and enum variant names
; BUG: matches constants in enum value and bit size number
; (maybe be a skill issue, maybe a grammar update is needed)
(field
  (identifier) @identifier)
(bit_field_declaration "::"
  (identifier) @identifier)
(enum_declaration "::"
  (identifier) @identifier)

; Strings
(string_content) @string
