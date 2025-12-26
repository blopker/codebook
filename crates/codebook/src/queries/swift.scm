(comment) @comment
(multiline_comment) @comment

(class_declaration
    name: (type_identifier) @identifier)

(function_declaration
    name: (simple_identifier) @identifier)

(protocol_declaration
    name: (type_identifier) @identifier)

(property_declaration
    name: (pattern) @identifier)

(parameter
    name: (simple_identifier) @identifier)

(line_string_literal) @string
(multi_line_string_literal) @string
