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
    ] @identifier))

(flow_mapping
  (_
    key: (flow_node
      [
        (double_quote_scalar)
        (single_quote_scalar)
      ] @identifier)))
