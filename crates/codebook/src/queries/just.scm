(comment) @comment

; Strings in settings, imports, and modules are file paths or shell
; configuration, not prose. Strings inside recipe bodies are covered by the
; injected language below.
((string) @string
  (#not-has-ancestor? @string "recipe_body" "setting" "import" "module"))

(recipe_header name: (identifier) @identifier.function)
(parameter name: (identifier) @identifier.parameter)
(assignment left: (identifier) @identifier.variable)
(alias left: (identifier) @identifier)
(module name: (identifier) @identifier.module)

; Recipe bodies without a shebang run in the default shell.
(recipe_body
  !shebang) @injection.bash

; Backtick commands evaluate in a shell. Skip ones inside recipe bodies;
; the body injection already covers those bytes.
((external_command
  (command_body) @injection.bash)
  (#not-has-ancestor? @injection.bash "recipe_body"))

; Shebang recipes: read the language from the shebang line. Each line is
; injected separately so the shebang itself isn't spell-checked.
(recipe_body
  (shebang
    (language) @injection.language)
  (recipe_line) @injection.content)
