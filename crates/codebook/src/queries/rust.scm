; Function names - excluding trait implementations (issue #225)
; Functions in non-trait impl blocks
(impl_item
    !trait
    body: (declaration_list
        (function_item
            name: (identifier) @identifier)))
; Functions in trait definitions
(trait_item
    body: (declaration_list
        (function_item
            name: (identifier) @identifier)))
; Top-level functions
(source_file (function_item name: (identifier) @identifier))
; Functions in modules
(mod_item body: (declaration_list (function_item name: (identifier) @identifier)))
; Nested functions (inside blocks)
(block (function_item name: (identifier) @identifier))
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
