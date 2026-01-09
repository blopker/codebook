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

; Fields and enum variant names
(field
  (identifier) @identifier)
(bit_field_member
  name: (identifier) @identifier)
(enum_member
  name: (identifier) @identifier)

; Strings
(string_content) @string
