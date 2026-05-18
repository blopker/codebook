; Comments - capture comment content for spell checking
(line_comment
  (comment_content) @comment.line)
(block_comment
  (comment_content) @comment.block)

; String literals
(string_literal) @string

; Entity declarations
(entity_declaration
  entity: (identifier) @identifier.type)

; Architecture definitions
(architecture_definition
  architecture: (identifier) @identifier.type)

; Signal declarations
(signal_declaration
  (identifier_list
    (identifier) @identifier.variable))

; Variable declarations
(variable_declaration
  (identifier_list
    (identifier) @identifier.variable))

; Constant declarations
(constant_declaration
  (identifier_list
    (identifier) @identifier.constant))

; Function specifications
(function_specification
  function: (identifier) @identifier.function)

; Procedure specifications
(procedure_specification
  procedure: (identifier) @identifier.function)

; Component declarations
(component_declaration
  component: (identifier) @identifier.type)

; Type declarations
(type_declaration
  type: (identifier) @identifier.type)

; Subtype declarations
(subtype_declaration
  type: (identifier) @identifier.type)

; Port/generic interface declarations
(interface_declaration
  (identifier_list
    (identifier) @identifier.parameter))
(interface_signal_declaration
  (identifier_list
    (identifier) @identifier.parameter))
(interface_variable_declaration
  (identifier_list
    (identifier) @identifier.parameter))
(interface_constant_declaration
  (identifier_list
    (identifier) @identifier.parameter))

; Labels
(label) @identifier

; Alias declarations
(alias_declaration
  (identifier) @identifier.variable)
