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
] @string.special.symbol
(comment) @comment

(alias) @identifier

(call
    (arguments
        (identifier) @identifier))

(call
    (identifier) @identifier)

(binary_operator
    left: (identifier) @identifier)
