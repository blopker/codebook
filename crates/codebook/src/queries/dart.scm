(comment) @comment

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
(initialized_variable_definition name: (identifier) @identifier.variable)
(static_final_declaration name: (identifier) @identifier.variable)
(enum_constant name: (identifier) @identifier.constant)

; Variables (class fields)
(initialized_identifier (identifier) @identifier.variable)

; Parameters
(formal_parameter name: (identifier) @identifier.parameter)

; Import aliases
(import_specification alias: (identifier) @identifier.module)

; String content (excludes interpolation expressions)
(template_chars_single_single) @string
(template_chars_double_single) @string
(template_chars_single) @string
(template_chars_double) @string
