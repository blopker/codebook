; Comments - capture LaTeX comments (lines starting with %)
(line_comment) @comment

; Prose. `word` is the leaf node of prose, so each word is captured exactly
; once, however deeply commands nest. Arguments that hold identifiers, paths,
; options, or math rather than prose are skipped by their ancestor. The
; `curly_group_text*` kinds only ever hold names (environments, colours,
; acronyms, citation keys). The name-holding kinds are repeated in the
; `\text{…}` pattern below; keep the two in sync.
((word) @string
  (#not-has-ancestor? @string
    "curly_group_text" "curly_group_text_list"
    "label_definition" "label_reference" "label_reference_range"
    "class_include" "package_include"
    "graphics_include" "svg_include" "inkscape_include"
    "latex_include" "import_include" "verbatim_include"
    "biblatex_include" "bibtex_include" "bibstyle_include"
    "tikz_library_import"
    "inline_formula" "displayed_equation" "math_environment"
    "new_command_definition" "old_command_definition" "let_command_definition"
    "environment_definition" "paired_delimiter_definition"
    "color_definition" "color_set_definition"
    "acronym_reference" "glossary_entry_reference"
    "brack_group_key_value" "curly_group_key_value"))

; `\text{…}` and `\intertext{…}` hold prose inside math, which the pattern
; above skips. Its `#not-has-ancestor?` list repeats the name-holding kinds
; from the pattern above; keep the two in sync.
((word) @string
  (#has-ancestor? @string "text_mode")
  (#has-ancestor? @string "inline_formula" "displayed_equation" "math_environment")
  (#not-has-ancestor? @string
    "curly_group_text" "curly_group_text_list"
    "label_definition" "label_reference" "label_reference_range"
    "brack_group_key_value" "curly_group_key_value"))

; An environment's optional argument is prose when it's a title
; (`\begin{theorem}[Main result]`), but not when it's a float placement.
((begin options: (brack_group) @skip)
  (#match? @skip "^\\[[htbpH!]+\\]$"))

; A colour command's model and spec (`\colorbox[rgb]{1,0,0}{text}`) aren't
; prose; its text argument is.
(color_reference model: (_) @skip)
(color_reference spec: (_) @skip)

; Label names are chosen where they're defined (`\label{eq:energy}`); a
; `\ref` only points at one, so a typo is reported once, at the definition.
(label_definition
  (curly_group_label (label) @identifier))
