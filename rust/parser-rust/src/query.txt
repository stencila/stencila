(source_file (line_comment) @comment)

(source_file
    (const_item
        name: (identifier) @name
    )
)
(source_file 
    (static_item
        name: (identifier) @name
    )
)
(source_file 
    (let_declaration
        pattern: (identifier) @name
    )
)


((identifier) @name)
