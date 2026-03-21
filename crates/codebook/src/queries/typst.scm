(comment) @comment

(text) @string

; Strings in math formulas
(formula (string) @string)

; Strings with attachments (superscript/subscript) in math
(attach (string) @string)

; Strings in content/dictionary values
(tagged (string) @string)

; Strings in groups (parenthesized expressions)
(group (string) @string)

(label) @identifier

; Import rename
(as (ident) @identifier.variable)

; Collect remainder of array in a new variable
(elude (ident) @identifier.variable)

(let pattern: (ident) @identifier.variable)
; Destructuring assignment
(let pattern: (group (ident) @identifier.variable))
; Destructuring dict into a new variable
(let pattern: (group (tagged field: (ident) (ident) @identifier.variable)))
; Dictionary type
(let pattern: (ident) value: (group (tagged field: (ident) @identifier.field)))

(for pattern: (ident) @identifier.variable)
(for pattern: (group (ident) @identifier.variable))

; Function
(let pattern: (call item: (ident) @identifier.function))
(let pattern: (call item: (ident) (group (ident) @identifier.parameter)))
(let pattern: (call item: (ident) (group (tagged field: (ident) @identifier.parameter))))

; Anonymous function
(lambda pattern: (ident) @identifier.parameter)
(lambda pattern: (group (ident) @identifier.parameter))
(lambda pattern: (group (group (ident) @identifier.parameter)))
