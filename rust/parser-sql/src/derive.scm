; A set of Tree-sitter queries used when deriving properties of parameters
; based on the column types and checks in CREATE TABLE statements

 (create_table_statement
    (table_parameters
        (table_column
            name: (identifier) @name
            type: (type (identifier) @type)

            ; capture all of the null_constraint including potential leading NOT
            ((null_constraint) @nullable)?

            ; only capture default_clause if it uses a literal
            (default_clause [
                ((number) @default)
                ((string content: ((_) @default)))
            ])?

            ; capture most common forms of check constraints involving the column
            (check_constraint [
                ; e.g. col_a < 10
                (binary_expression
                    left: [
                        (identifier) @check.identifier
                        (function_call
                            ; Previously we captured the function and identifier as below.
                            ; However, for some reason that is very slow (~ 3 times slower)
                            ; in both --debug and --release tests. So do simple "parsing in
                            ; rust instead.
                            ; function: (identifier) @check.function
                            ; arguments: (identifier) @check.identifier
                        ) @check.call
                    ]
                    right: [
                        ((number) @check.number)
                        ((string content: ((_) @check.string)))
                        ((type_cast . (string content: ((_) @check.string))))
                    ]
                ) @check
                
                ; e.g. col_a > 10 AND col_a <= 20
                (boolean_expression [
                    (binary_expression
                        left: [
                            (identifier) @check.identifier
                            (function_call) @check.call
                        ]
                        right: [
                            ((number) @check.number)
                            ((string content: ((_) @check.string)))
                            ((type_cast . (string content: ((_) @check.string))))
                        ]
                    ) @check
                ]) @checks
            ])?
        )
    )
 )