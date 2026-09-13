(comment) @comment

; Strings
(string_expression) @string
(indented_string_expression (string_fragment) @string.heredoc) 
[(path_expression) (hpath_expression) (spath_expression)] @string.special
(uri_expression) @string.special

; Parameters
(function_expression 
    universal: (identifier) @identifier.parameter)
(formal 
    name: (identifier) @identifier.parameter)

; Fields
(inherit 
    attrs: (inherited_attrs attr: (identifier) @identifier.field))
(inherit_from 
    attrs: (inherited_attrs attr: (identifier) @identifier.field))

; Constants, let binds are immutable, don't fall into variables category
(binding 
    attrpath: (attrpath (identifier)) @identifier.constant)

; Function
(apply_expression 
    function: [
        (variable_expression (identifier)) @identifier.function 
        (select_expression 
            attrpath: (attrpath attr: (identifier) @identifier.function .))])
