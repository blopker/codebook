; Playground: https://elixir-lang.org/tree-sitter-elixir/
; Quoted text is captured by its `quoted_content` pieces, which sit beside
; any `#{...}` interpolation, so interpolated code is only captured by the
; patterns for code.
(string (quoted_content) @string)
(charlist (quoted_content) @string)
[
  (atom)
  (keyword)
] @string.special
(quoted_atom (quoted_content) @string.special)
(quoted_keyword (quoted_content) @string.special)
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
