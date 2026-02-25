; Comments - capture comment content for spell checking
(line_comment
  (comment_content) @comment)
(block_comment
  (comment_content) @comment)

; String literals
(string_literal) @string

; Entity declarations
(entity_declaration
  entity: (identifier) @identifier)

; Architecture definitions
(architecture_definition
  architecture: (identifier) @identifier)

; Signal declarations
(signal_declaration
  (identifier_list
    (identifier) @identifier))

; Variable declarations
(variable_declaration
  (identifier_list
    (identifier) @identifier))

; Constant declarations
(constant_declaration
  (identifier_list
    (identifier) @identifier))

; Function specifications
(function_specification
  function: (identifier) @identifier)

; Procedure specifications
(procedure_specification
  procedure: (identifier) @identifier)

; Component declarations
(component_declaration
  component: (identifier) @identifier)

; Type declarations
(type_declaration
  type: (identifier) @identifier)

; Subtype declarations
(subtype_declaration
  type: (identifier) @identifier)

; Port/generic interface declarations
(interface_declaration
  (identifier_list
    (identifier) @identifier))
(interface_signal_declaration
  (identifier_list
    (identifier) @identifier))
(interface_variable_declaration
  (identifier_list
    (identifier) @identifier))
(interface_constant_declaration
  (identifier_list
    (identifier) @identifier))

; Labels
(label) @identifier

; Alias declarations
(alias_declaration
  (identifier) @identifier)
