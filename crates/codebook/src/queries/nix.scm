; Comments
(comment) @comment

; Strings. Only the literal fragments: `${ ... }` interpolations hold
; expressions, and capturing the whole string would overlap the identifier
; captures inside them.
(string_expression (string_fragment) @string)
(indented_string_expression (string_fragment) @string.heredoc)

; Paths and URIs
[
  (path_expression)
  (hpath_expression)
  (spath_expression)
  (uri_expression)
] @string.special

; Function parameters, including the `{ a, b }` destructuring form
(function_expression
  universal: (identifier) @identifier.parameter)

(formal
  name: (identifier) @identifier.parameter)

; `inherit foo;` / `inherit (expr) foo;` name bindings the file itself spells out
(inherit
  attrs: (inherited_attrs attr: (identifier) @identifier.field))

(inherit_from
  attrs: (inherited_attrs attr: (identifier) @identifier.field))

; Attribute bindings are Nix's definition site: attrset keys, `let` bindings,
; and the `foo = args: body` form that names functions.
(binding
  attrpath: (attrpath attr: (identifier) @identifier.constant))
