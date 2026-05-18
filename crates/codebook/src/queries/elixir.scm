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

(binary_operator
    left: (identifier) @identifier.variable)
