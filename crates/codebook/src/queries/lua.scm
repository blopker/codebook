; Comments
(comment) @comment

; Strings
(string) @string

; Function declarations
(function_declaration
  name: (identifier) @identifier.function)

(function_declaration
    (method_index_expression
        method: (identifier) @identifier.function))

; Variable assignments
(assignment_statement
  (variable_list
    (identifier) @identifier.variable))

(assignment_statement
    (variable_list
        (dot_index_expression
            field: (identifier) @identifier.field)))

; Function parameters
(parameters
  (identifier) @identifier.parameter)

; Table fields
(field
  name: (identifier) @identifier.field)
