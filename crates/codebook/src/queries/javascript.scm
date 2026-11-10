(comment) @comment
(string_fragment) @string
; Module specifiers name other packages, not the file's own words.
(import_statement source: (string) @skip)
(export_statement source: (string) @skip)
(call_expression
    function: (import)
    arguments: (arguments [(string) (template_string)] @skip))
((call_expression
    function: (identifier) @_fn
    arguments: (arguments [(string) (template_string)] @skip))
  (#eq? @_fn "require"))
(variable_declarator
    name: (identifier) @identifier.variable)
(object
    (pair
    key: (property_identifier) @identifier.field))
(catch_clause
    parameter: (identifier) @identifier.parameter)
(jsx_text) @string
(shorthand_property_identifier) @identifier.field
(function_declaration
    name: (identifier) @identifier.function)
(function_declaration
    parameters: (formal_parameters
    (identifier) @identifier.parameter))
(method_definition
    name: (property_identifier) @identifier.function)
(class_declaration
    name: (identifier) @identifier.type)
