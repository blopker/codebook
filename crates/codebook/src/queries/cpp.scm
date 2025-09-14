(comment) @comment

(preproc_def
  name: (identifier) @identifier)

(type_definition
  declarator: (type_identifier) @identifier)

(struct_specifier
  name: (type_identifier) @identifier)

(class_specifier
  name: (type_identifier) @identifier)

(field_declaration
  declarator: (field_identifier) @identifier)

(pointer_declarator
  declarator: (field_identifier) @identifier)

(enum_specifier
  name: (type_identifier) @identifier)

(enumerator
  name: (identifier) @identifier)

(init_declarator
  declarator: (identifier) @identifier)

(pointer_declarator
  declarator: (identifier) @identifier)

(init_declarator
  (string_literal
    (string_content) @string_content))

(function_declarator
  declarator: (identifier) @identifier)

(parameter_declaration
  declarator: (identifier) @identifier)

(call_expression
  (argument_list
    (string_literal
      [(string_content) (escape_sequence)] @string)))

(binary_expression
    (string_literal
        (string_content) @string))

(concatenated_string
    (string_literal
        (string_content) @string))

(initializer_list
    (string_literal
        (string_content) @string))
