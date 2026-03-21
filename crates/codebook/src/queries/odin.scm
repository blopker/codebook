; Comments
(comment) @comment.line
(block_comment) @comment.block

; Procedure declarations (including parameter names)
(procedure_declaration
  (expression) @identifier.function)
(overloaded_procedure_declaration
  (expression) @identifier.function)
(parameter
  (identifier) @identifier.parameter)
(default_parameter
  (identifier) @identifier.parameter)

; Variables and constants identifiers (declaration-only)
(var_declaration
  (expression) @identifier.variable ":")
(assignment_statement
  (expression) @identifier.variable ":=")
(const_declaration
  (expression)+ @identifier.constant)
(const_type_declaration
  (expression)+ @identifier.constant)

; Struct, enum, union, bit_fields names
(struct_declaration
  (expression) @identifier.type)
(enum_declaration
  (expression) @identifier.type)
(union_declaration
  (expression) @identifier.type)
(bit_field_declaration
  (expression) @identifier.type "::")

; Fields and enum variant names
(field
  (identifier) @identifier.field)
(bit_field_member
  name: (identifier) @identifier.field)
(enum_member
  name: (identifier) @identifier.constant)

; Strings
(string_content) @string
