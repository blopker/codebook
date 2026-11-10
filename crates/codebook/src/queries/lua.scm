; Comments
(comment) @comment

; Strings
(string) @string
; `require` names another module, not the file's own words.
((function_call
    name: (identifier) @_fn
    arguments: (arguments (string) @skip))
  (#eq? @_fn "require"))

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
