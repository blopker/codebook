; Comments
(comment) @comment

; String content
(string_content) @string

; Variable declarations (const/var declarations)
(variable_declaration
  (identifier) @identifier.variable)

; Function declarations
(function_declaration
  (identifier) @identifier.function)

; Function parameters
(parameter
  (identifier) @identifier.parameter)

; Payload identifiers (capture variables in for/while loops)
(payload
  (identifier) @identifier.variable)

(struct_declaration
  (container_field
    (identifier) @identifier.field))
