(comment) @comment

(string_content) @string

(function_definition
    name: (identifier) @identifier)

(class_definition
    name: (identifier) @identifier)

(assignment
    (identifier) @identifier)

(parameters
  (identifier) @identifier)

; Matches typed parameters (e.g., "name: str")
; The identifier for the name is a *direct child* of typed_parameter,
; while the type identifier is nested inside a (type) node.
(typed_parameter
  (identifier) @identifier)

; Matches parameters with default values (e.g., "limit=10")
(default_parameter
  (identifier) @identifier)

; Matches typed parameters with default values (e.g., "limit: int = 10")
(typed_default_parameter
  (identifier) @identifier)
