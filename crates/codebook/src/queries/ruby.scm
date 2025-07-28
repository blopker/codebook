(string) @string
(comment) @comment
(assignment (identifier) @identifier)
(method
    (method_parameters (keyword_parameter (identifier) @identifier)))
(method
    (method_parameters (identifier) @identifier))
(method name: (identifier) @identifier)
(heredoc_body
    (heredoc_content) @string
    (heredoc_end) @language
    (#downcase! @language))
