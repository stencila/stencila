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
