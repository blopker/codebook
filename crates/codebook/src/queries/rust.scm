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
; Bindings in pattern positions that are not in function or closure parameters.
; Drop names starting with an uppercase letter because the grammar can't tell a
; binding from a constant or unit variant such as `MAX` or `None`.
((_pattern/identifier) @identifier.variable
  (#not-match? @identifier.variable "^\\p{Lu}")
  (#not-has-ancestor? @identifier.variable "parameter" "closure_parameters"))
; Uppercase bindings not in function or closure parameters, in pattern positions
; where a name is always a binding
([
    (let_declaration pattern: (identifier) @identifier.variable)
    (for_expression pattern: (identifier) @identifier.variable)
    (ref_pattern (identifier) @identifier.variable)
    (mut_pattern (identifier) @identifier.variable)
  ]
  (#match? @identifier.variable "^\\p{Lu}")
  (#not-has-ancestor? @identifier.variable "parameter" "closure_parameters"))
; Bindings in pattern positions that are in function or closure parameters.
; Parameter patterns must be irrefutable, so consider everything to be a binding
; regardless of case.
((_pattern/identifier) @identifier.parameter
  (#has-ancestor? @identifier.parameter "parameter" "closure_parameters"))
; The name in `name @ subpattern`. It's always a binding, so check it regardless
; of case.
((captured_pattern . (identifier) @identifier.variable)
  (#not-has-ancestor? @identifier.variable "parameter" "closure_parameters"))
((captured_pattern . (identifier) @identifier.parameter)
  (#has-ancestor? @identifier.parameter "parameter" "closure_parameters"))
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
; Use declarations with renaming
(use_as_clause
    alias: (identifier) @identifier)
; Crate aliases
(extern_crate_declaration
    alias: (identifier) @identifier.module)
(block_comment) @comment.block
(line_comment) @comment.line
(string_content) @string
