; Capture YAML comments and scalar values for spell-checking
(comment) @comment

; Unquoted plain scalars
(plain_scalar) @string

; Single- and double-quoted scalars
(single_quote_scalar) @string
(double_quote_scalar) @string

; Block scalars (literal '|' and folded '>')
(block_scalar) @string

; Capture mapping keys as identifiers (useful for keys that are plain scalars)
(block_mapping_pair
  key: (flow_node
    [
      (double_quote_scalar)
      (single_quote_scalar)
    ] @identifier.field))

(flow_mapping
  (_
    key: (flow_node
      [
        (double_quote_scalar)
        (single_quote_scalar)
      ] @identifier.field)))

; CI scripts (GitHub Actions `run:`, GitLab `script:` and friends) are shell,
; checked with the Bash grammar: command names and flags aren't words, but
; the script's comments and strings are. The injection owns the text, so the
; scalar patterns above don't also check it.
((block_mapping_pair
  key: (flow_node (plain_scalar (string_scalar) @_key))
  value: [
    (block_node (block_scalar) @injection.bash)
    (flow_node (plain_scalar) @injection.bash)
    (block_node (block_sequence (block_sequence_item (flow_node (plain_scalar) @injection.bash))))
  ])
  (#any-of? @_key "run" "script" "before_script" "after_script"))
