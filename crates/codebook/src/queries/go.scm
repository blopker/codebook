(comment) @comment
(function_declaration (identifier) @func_declaration)
(var_spec (identifier) @var_spec)
(short_var_declaration
    (expression_list (identifier) @short_var))
(parameter_declaration
  name: (identifier) @parameter_name)
(field_identifier) @field
(type_identifier) @type_name
(import_spec
  name: (package_identifier) @import_alias)
(package_clause (package_identifier) @package_name)
(label_name) @label
(field_declaration tag: (raw_string_literal) @struct_tag)
(const_spec name: (identifier) @const_name)
(range_clause left: (expression_list (identifier) @range_var))
(interpreted_string_literal) @string_literal
(array_type (interpreted_string_literal) @array_string)
