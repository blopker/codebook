(comment) @comment
(string_content) @string
(function_definition
    name: (word) @identifier.function)
(heredoc_body) @string.heredoc
(variable_assignment
    name: (variable_name) @identifier.variable)
