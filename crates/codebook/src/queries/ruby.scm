(string) @string
(comment) @comment
(assignment (identifier) @identifier.variable)
(method
    (method_parameters (keyword_parameter (identifier) @identifier.parameter)))
(method
    (method_parameters (identifier) @identifier.parameter))
(method name: (identifier) @identifier.function)
(heredoc_body
    (heredoc_content) @string.heredoc
    (heredoc_end) @language
    (#downcase! @language))
