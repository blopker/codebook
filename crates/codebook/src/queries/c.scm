(comment) @comment
(preproc_def
    name: (identifier) @identifier.constant)
(type_definition
    declarator: (type_identifier) @identifier.type)
(struct_specifier
    name: (type_identifier) @identifier.type)
(field_declaration
    declarator: (field_identifier) @identifier.field)
(pointer_declarator
    declarator: (field_identifier) @identifier.field)
(enum_specifier
    name: (type_identifier) @identifier.type)
(enumerator
    name: (identifier) @identifier.constant)
(init_declarator
    declarator: (identifier) @identifier.variable)
(pointer_declarator
    declarator: (identifier) @identifier.variable)
(declaration
    declarator: (identifier) @identifier.variable)
(init_declarator
    (string_literal
        (string_content) @string))
(function_declarator
    declarator: (identifier) @identifier.function)
(parameter_declaration
    declarator: (identifier) @identifier.parameter)
    (call_expression
        (argument_list
            (string_literal
                [(string_content) (escape_sequence)] @string)))
