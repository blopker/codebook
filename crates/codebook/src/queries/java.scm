[
    (line_comment)
    (block_comment)
] @comment
[
    (character_literal)
    (string_literal)
] @string
(variable_declarator
    name: (identifier) @identifier)
(interface_declaration
    name: (identifier) @identifier)
(class_declaration
    name: (identifier) @identifier)
(method_declaration
    name: (identifier) @identifier)
(enum_declaration
    name: (identifier) @identifier)
(enum_constant
    name: (identifier) @identifier)
(formal_parameter
    name: (identifier) @identifier)
(catch_formal_parameter
    name: (identifier) @identifier)
