; Comments
(comment) @comment

; Strings
(string) @string

; Function declarations
(function_declaration
  name: (identifier) @identifier)

(function_declaration
    (method_index_expression
        method: (identifier) @identifier))

; Variable assignments
(assignment_statement
  (variable_list
    (identifier) @identifier))

(assignment_statement
    (variable_list
        (dot_index_expression
            field: (identifier) @identifier)))

; Function parameters
(parameters
  (identifier) @identifier)

; Table fields
(field
  name: (identifier) @identifier)
