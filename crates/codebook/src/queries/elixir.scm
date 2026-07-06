; Playground: https://elixir-lang.org/tree-sitter-elixir/
[
  (string)
  (charlist)
] @string
[
  (atom)
  (quoted_atom)
  (keyword)
  (quoted_keyword)
] @string.special
(comment) @comment

(alias) @identifier.type

(call
    (arguments
        (identifier) @identifier))

(call
    (identifier) @identifier.function)

; Only match (=) and comprehension/for generators (<-) bind variables;
; the left side of any other binary operator (arithmetic, pipes, ...) is a
; usage, not a definition.
(binary_operator
    left: (identifier) @identifier.variable
    operator: ["=" "<-"])
