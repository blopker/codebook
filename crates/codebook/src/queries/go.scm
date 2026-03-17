(comment) @comment
(function_declaration (identifier) @identifier.function)
(var_spec (identifier) @identifier.variable)
(short_var_declaration
    (expression_list (identifier) @identifier.variable))
(parameter_declaration
  name: (identifier) @identifier.parameter)
(field_identifier) @identifier.field
(type_identifier) @identifier.type
(import_spec
  name: (package_identifier) @identifier.module)
(package_clause (package_identifier) @identifier.module)
(label_name) @identifier
(field_declaration tag: (raw_string_literal) @string.special)
(const_spec name: (identifier) @identifier.constant)
(range_clause left: (expression_list (identifier) @identifier.variable))
(interpreted_string_literal) @string
(array_type (interpreted_string_literal) @string)
