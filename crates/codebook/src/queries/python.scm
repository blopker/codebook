(comment) @comment
(string_content) @string
(function_definition
    name: (identifier) @identifier)
(function_definition
    parameters: (parameters) @identifier)
(class_definition
    name: (identifier) @identifier)
(assignment
    left: (identifier) @identifier)
(import_statement
    name: (aliased_import
        alias: (identifier) @identifier))
(import_from_statement
    name: (aliased_import
        alias: (identifier) @identifier))
