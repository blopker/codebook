; HTML comments
(comment) @comment

; Visible text nodes between HTML tags
(text) @string

; Quoted attribute values — e.g. alt="descriptoin", title="Wellcome"
(quoted_attribute_value) @string

; JavaScript / TypeScript inside <script> and <script lang="ts"> blocks.
; Captured as text so string literals and identifiers are spell-checked.
(script_element
  (raw_text) @string)

; CSS / SCSS inside <style> and <style lang="scss"> blocks.
(style_element
  (raw_text) @string)
