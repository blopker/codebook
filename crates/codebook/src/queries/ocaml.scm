; Comments
[(comment) (line_number_directive) (directive) (shebang)] @comment

; Strings
[(string) (character)] @string
(quoted_string (quoted_string_content) @string.heredoc) @string
(conversion_specification) @string.special
(pretty_printing_indication) @string.special
(tag) @string.special

; Identifiers
(attribute_id) @identifier
(let_binding pattern: (value_name) @identifier)

; Constants
(constructor_name) @identifier.constant
(boolean) @identifier.constant

; Parameters - only in function parameter position
(parameter pattern: (typed_pattern pattern: (value_pattern) @identifier.parameter))
(parameter pattern: (value_pattern) @identifier.parameter)

; Fields
[(label_name) (field_name) (instance_variable_name)] @identifier.field

; Val signatures and externals
(value_specification (value_name) @identifier.function)
(external (value_name) @identifier.function)

; Method definitions
(method_definition (method_name) @identifier.function)

; Types
[(class_name) (class_type_name) (type_constructor)] @identifier.type
(type_variable) @identifier.type

; Modules
[(module_name) (module_type_name)] @identifier.module
