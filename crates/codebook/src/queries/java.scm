[
    (line_comment)
    (block_comment)
] @comment
[
    (character_literal)
    (string_literal)
] @string
(variable_declarator
    name: (identifier) @identifier.variable)
(interface_declaration
    name: (identifier) @identifier.type)
(class_declaration
    name: (identifier) @identifier.type)
(method_declaration
    name: (identifier) @identifier.function)
(enum_declaration
    name: (identifier) @identifier.type)
(enum_constant
    name: (identifier) @identifier.constant)
(formal_parameter
    name: (identifier) @identifier.parameter)
(catch_formal_parameter
    name: (identifier) @identifier.parameter)
