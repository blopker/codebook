; Function names - excluding trait implementations (issue #225)
; Functions in non-trait impl blocks
(impl_item
    !trait
    body: (declaration_list
        (function_item
            name: (identifier) @identifier.function)))
; Functions in trait definitions with default bodies
(trait_item
    body: (declaration_list
        (function_item
            name: (identifier) @identifier.function)))
; Functions in trait definitions without default bodies
(trait_item
    body: (declaration_list
        (function_signature_item
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
(const_item
    name: (identifier) @identifier.constant)
; Skip associated consts in trait impls because they're named by the trait
(impl_item
    trait: (_)
    body: (declaration_list
        (const_item
            name: (identifier) @skip)))
; Tag statics as constants because they're named like and mostly used as
; constants. Drop statics in extern blocks because they're named by the foreign
; library.
((static_item
    name: (identifier) @identifier.constant)
  (#not-has-ancestor? @identifier.constant "foreign_mod_item"))
(struct_item
    name: (type_identifier) @identifier.type)
(field_declaration
    name: (field_identifier) @identifier.field)
(enum_item
    name: (type_identifier) @identifier.type)
(enum_variant
    name: (identifier) @identifier.constant)
(union_item
    name: (type_identifier) @identifier.type)
(trait_item
    name: (type_identifier) @identifier.type)
; Type aliases
(type_item
    name: (type_identifier) @identifier.type)
; Associated types in trait definitions
(trait_item
    body: (declaration_list
        (associated_type
            name: (type_identifier) @identifier.type)))
; Skip associated types in trait impls because they're named by the trait
(impl_item
    trait: (_)
    body: (declaration_list
        (type_item
            name: (type_identifier) @skip)))
(mod_item
    name: (identifier) @identifier.module)
(block_comment) @comment.block
(line_comment) @comment.line
(string_content) @string
