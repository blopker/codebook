(comment) @comment

(string_content) @string

(function_definition
    name: (identifier) @identifier.function)

(class_definition
    name: (identifier) @identifier.type)

(assignment
    left: (identifier) @identifier.variable)

(import_statement
    name: (aliased_import
        alias: (identifier) @identifier.module))

(import_from_statement
    name: (aliased_import
        alias: (identifier) @identifier.module))

(parameters
  (identifier) @identifier.parameter)

; Matches typed parameters (e.g., "name: str")
; The identifier for the name is a *direct child* of typed_parameter,
; while the type identifier is nested inside a (type) node.
(typed_parameter
  (identifier) @identifier.parameter)

; Matches parameters with default values (e.g., "limit=10")
(default_parameter
  (identifier) @identifier.parameter)

; Matches typed parameters with default values (e.g., "limit: int = 10")
(typed_default_parameter
  (identifier) @identifier.parameter)
