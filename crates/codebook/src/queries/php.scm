; Comments
(comment) @comment

; Strings
(string_content) @string

; Names (covers function names, class names, etc.)
(class_declaration
    name: (name) @identifier)
(const_declaration
    (const_element (name) @identifier))
(namespace_definition
    (namespace_name (name) @identifier))
(property_element
    (variable_name (name) @identifier))
(method_declaration
    name: (name) @identifier)
(assignment_expression
    left: (variable_name (name) @identifier))
(function_definition
    name: (name) @identifier)
(simple_parameter
    (variable_name (name) @identifier))
(catch_clause
    (variable_name (name) @identifier))
