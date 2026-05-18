(comment) @comment
(string) @string

(var) @identifier.variable

(function_clause
  name: (atom) @identifier.function)

; Atoms in specific contexts (avoids overlap with function names above)
(module_attribute (atom) @string.special)
(tuple (atom) @string.special)
(map_field (atom) @string.special)
(call (atom) @string.special)
