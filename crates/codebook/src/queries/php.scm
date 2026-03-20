; Comments
(comment) @comment

; Strings
(string_content) @string

; Names (covers function names, class names, etc.)
(class_declaration
    name: (name) @identifier.type)
(const_declaration
    (const_element (name) @identifier.constant))
(namespace_definition
    (namespace_name (name) @identifier.module))
(property_element
    (variable_name (name) @identifier.field))
(method_declaration
    name: (name) @identifier.function)
(assignment_expression
    left: (variable_name (name) @identifier.variable))
(function_definition
    name: (name) @identifier.function)
(simple_parameter
    (variable_name (name) @identifier.parameter))
(catch_clause
    (variable_name (name) @identifier.parameter))
