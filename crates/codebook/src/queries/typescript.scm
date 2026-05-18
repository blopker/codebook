(comment) @comment
(string_fragment) @string
(variable_declarator
    name: (identifier) @identifier.variable)
(object
    (pair
    key: (property_identifier) @identifier.field))
(interface_declaration
    name: (type_identifier) @identifier.type)
(interface_body
    (property_signature
        name: (property_identifier) @identifier.field))
(catch_clause
    parameter: (identifier) @identifier.parameter)
(jsx_text) @string
(shorthand_property_identifier) @identifier.field
(function_declaration
    name: (identifier) @identifier.function)
(formal_parameters
    (required_parameter
    pattern: (identifier) @identifier.parameter))
(formal_parameters
    (optional_parameter
    pattern: (identifier) @identifier.parameter))
(method_definition
    name: (property_identifier) @identifier.function)
(class_declaration
    name: (type_identifier) @identifier.type)
(public_field_definition
    name: (property_identifier) @identifier.field)
