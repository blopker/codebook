(comment) @comment.line
(multiline_comment) @comment.block

(class_declaration
    name: (type_identifier) @identifier.type)

(function_declaration
    name: (simple_identifier) @identifier.function)

(protocol_declaration
    name: (type_identifier) @identifier.type)

(property_declaration
    name: (pattern) @identifier.field)

(parameter
    name: (simple_identifier) @identifier.parameter)

(line_string_literal) @string
(multi_line_string_literal) @string
