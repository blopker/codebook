; Doc comments (`///`, `/** */`) parse as their own `documentation_comment`
; node rather than as `comment`, so both kinds are needed to cover every
; comment form. Capturing the whole `documentation_comment` also covers the
; `documentation_block_comment` it wraps.
[(comment) (documentation_comment)] @comment

; Types
(class_declaration name: (identifier) @identifier.type)
(enum_declaration name: (identifier) @identifier.type)
(mixin_declaration name: (identifier) @identifier.type)
(type_alias . _ . (type_identifier) @identifier.type)

; Functions
(function_signature name: (identifier) @identifier.function)
(getter_signature name: (identifier) @identifier.function)
(setter_signature name: (identifier) @identifier.function)

; Variables (local)
(initialized_variable_declaration name: (identifier) @identifier.variable)
(static_final_declaration name: (identifier) @identifier.variable)
(enum_entry name: (identifier) @identifier.constant)

; Fields (class/mixin instance variables)
(initialized_identifier (identifier) @identifier.field)

; Parameters
(formal_parameter name: (identifier) @identifier.parameter)

; Import aliases
(import_specification alias: (identifier) @identifier.module)

; String content (excludes interpolation expressions)
(template_chars_single_single) @string
(template_chars_double_single) @string
(template_chars_single) @string
(template_chars_double) @string
