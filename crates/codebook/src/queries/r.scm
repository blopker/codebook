(comment) @comment
(string) @string

(parameter name: (identifier) @identifier)

(binary_operator
    lhs: (identifier) @identifier
    operator: ["<-" "="])
(binary_operator
    operator: "->"
    rhs: (identifier) @identifier)

;---------------------------------------
; Less clear-cut spell checking targets:
;---------------------------------------

; Functions with ... args sometimes use the argument names similarly to
; new variable definitions which should be spell-checked.
; e.g. dplyr::mutate(data_table, new_column_name=col_a + col_b) should check `new_column_name`
(argument  name: (identifier) @identifier)

; Assignments with `$` can similarly define new names
; For chains, only check the last name since the earlier names are not being newly defined
; e.g. `my_list$data_table$new_column_name <- 1 + 2` should check `new_column_name`
(binary_operator
  lhs: (extract_operator
          operator: "$"
          rhs: (identifier) @identifier)
  operator: ["<-" "="])

(binary_operator
  operator: "->"
  rhs: (extract_operator
          operator: "$"
          rhs: (identifier) @identifier))
