(comment) @comment
; packages
(package_clause name: (package_identifier) @identifier.module)
; vals
(val_definition pattern: (identifier) @identifier.variable)
(val_definition pattern: (tuple_pattern (identifier)) @identifier.variable)
(val_definition pattern: (case_class_pattern (identifier)) @identifier.variable)
(val_definition  pattern: (infix_pattern (identifier)) @identifier.variable)
; vars
(var_definition pattern: (identifier) @identifier.variable)
(var_definition pattern: (tuple_pattern (identifier)) @identifier.variable)
(var_definition pattern: (case_class_pattern (identifier)) @identifier.variable)
(var_definition  pattern: (infix_pattern (identifier)) @identifier.variable)
; functions
(function_definition name: (identifier) @identifier.function)
(function_definition parameters: (parameters (parameter name: (identifier) @identifier.parameter)))
(function_definition type_parameters: (type_parameters _ name: (identifier) @identifier.parameter))
; classes
(class_definition name: (identifier) @identifier.type)
(class_definition class_parameters: (class_parameters (class_parameter name: (identifier) @identifier.parameter)))
(class_definition type_parameters: (type_parameters _ name: (identifier) @identifier.parameter))
; traits
(trait_definition name: (identifier) @identifier.type)
(trait_definition type_parameters: (type_parameters _ name: (identifier) @identifier.parameter))
; type aliases
(type_definition name: (type_identifier) @identifier.type)
(type_definition type_parameters: (type_parameters _ name: (identifier) @identifier.parameter))
; pattern matching
(case_clause pattern: (identifier) @identifier.variable)
(case_clause pattern: (tuple_pattern (identifier)) @identifier.variable)
(case_clause pattern: (case_class_pattern (identifier)) @identifier.variable)
(case_clause pattern: (infix_pattern (identifier)) @identifier.variable)
