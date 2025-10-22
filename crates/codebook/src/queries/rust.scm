(function_item
    name: (identifier) @identifier)
(parameter
    pattern: (identifier) @identifier)
(let_declaration
    pattern: (identifier) @identifier)
(struct_item
    name: (type_identifier) @identifier)
(field_declaration
    name: (field_identifier) @identifier)
(block_comment) @comment
(line_comment) @comment
(string_content) @string
