; comment
((comment) @comment)

; assigns
(source_file
    [
        (create_view_statement
            . (identifier) @identifer
        )
        (create_table_statement
            . (identifier) @identifer
        )
    ]
)

; declares
(source_file
    (create_table_statement
        (identifier) @table
        (table_parameters
            (table_column
                name: (identifier) @column
            )
        )
    )
)

; uses
(select_statement
    [
        (from_clause
            . (identifier) @identifer
        )
        (join_clause
            . (identifier) @identifer
        )
    ]
)

; alters
[
    (insert_statement
        . (identifier) @identifer
    )
    (update_statement
        . (identifier) @identifer
    )
    (delete_statement
        . (from_clause
            (identifier) @identifer
        )
    )
    (drop_statement
        target: (identifier) @identifer
    )
]

; binding
((argument_reference) @index)
