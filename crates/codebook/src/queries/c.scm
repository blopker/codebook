(comment) @comment
(preproc_def
    name: (identifier) @identifier.constant)
(preproc_function_def
    name: (identifier) @identifier.function)
(preproc_params) @identifier.parameter
(type_definition
    declarator: (type_identifier) @identifier.type)
(struct_specifier
    name: (type_identifier) @identifier.type)
(union_specifier
    name: (type_identifier) @identifier.type)
(field_declaration
    declarator: (field_identifier) @identifier.field)
(pointer_declarator
    declarator: (field_identifier) @identifier.field)
(array_declarator
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
(array_declarator
    declarator: (identifier) @identifier.variable)
(function_declarator
    declarator: (identifier) @identifier.function)
(parameter_declaration
    declarator: (identifier) @identifier.parameter)
(string_content) @string
