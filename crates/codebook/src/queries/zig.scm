; Comments
(comment) @comment

; String content
(string_content) @string

; Variable declarations (const/var declarations)
(variable_declaration
  (identifier) @identifier)

; Function declarations
(function_declaration
  (identifier) @identifier)

; Function parameters
(parameter
  (identifier) @identifier)

; Payload identifiers (capture variables in for/while loops)
(payload
  (identifier) @identifier)

(struct_declaration
  (container_field
    (identifier) @identifier))
