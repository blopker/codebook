(comment) @comment

; Types
(class_declaration name: (identifier) @identifier.type)
(enum_declaration name: (identifier) @identifier.type)
(type_identifier) @identifier.type

; Functions
(function_signature name: (identifier) @identifier.function)
(getter_signature name: (identifier) @identifier.function)
(setter_signature name: (identifier) @identifier.function)

; Variables
(initialized_variable_definition name: (identifier) @identifier.variable)
(static_final_declaration name: (identifier) @identifier.variable)
(enum_constant name: (identifier) @identifier.constant)

; Parameters
(formal_parameter name: (identifier) @identifier.parameter)

; Import aliases
(import_specification alias: (identifier) @identifier.module)

; Strings in common expression contexts (imports excluded)
(argument (string_literal) @string)
(named_argument (string_literal) @string)
(initialized_variable_definition value: (string_literal) @string)
(static_final_declaration value: (string_literal) @string)
(additive_expression (string_literal) @string)
(equality_expression (string_literal) @string)
(return_statement (string_literal) @string)
(expression_statement (string_literal) @string)
(assignment_expression right: (string_literal) @string)
(list_literal (string_literal) @string)
(set_or_map_literal (string_literal) @string)
(pair (string_literal) @string)
(conditional_expression (string_literal) @string)
(switch_expression_case (string_literal) @string)
(switch_statement_case (string_literal) @string)
