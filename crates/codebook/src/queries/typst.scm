(comment) @comment

(text) @string

(string) @string

(label) @identifier

; Import rename
(as (ident) @identifier)

; Collect remainder of array in a new variable
(elude (ident) @identifier)

(let pattern: (ident) @identifier)
; Destructuring assignment
(let pattern: (group (ident) @identifier))
; Destructuring dict into a new variable
(let pattern: (group (tagged field: (ident) (ident) @identifier)))
; Dictionary type
(let pattern: (ident) value: (group (tagged field: (ident) @identifier)))

(for pattern: (ident) @identifier)
(for pattern: (group (ident) @identifier))

; Function
(let pattern: (call item: (ident) @identifier))
(let pattern: (call item: (ident) (group (ident) @identifier)))
(let pattern: (call item: (ident) (group (tagged field: (ident) @identifier))))

; Anonymous function
(lambda pattern: (ident) @identifier)
(lambda pattern: (group (ident) @identifier))
(lambda pattern: (group (group (ident) @identifier)))
