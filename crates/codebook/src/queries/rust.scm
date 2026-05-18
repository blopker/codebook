; Function names - excluding trait implementations (issue #225)
; Functions in non-trait impl blocks
(impl_item
    !trait
    body: (declaration_list
        (function_item
            name: (identifier) @identifier.function)))
; Functions in trait definitions
(trait_item
    body: (declaration_list
        (function_item
            name: (identifier) @identifier.function)))
; Top-level functions
(source_file (function_item name: (identifier) @identifier.function))
; Functions in modules
(mod_item body: (declaration_list (function_item name: (identifier) @identifier.function)))
; Nested functions (inside blocks)
(block (function_item name: (identifier) @identifier.function))
(parameter
    pattern: (identifier) @identifier.parameter)
(let_declaration
    pattern: (identifier) @identifier.variable)
(struct_item
    name: (type_identifier) @identifier.type)
(field_declaration
    name: (field_identifier) @identifier.field)
(block_comment) @comment.block
(line_comment) @comment.line
(string_content) @string
