(program
    [
        (variable_declaration
            (variable_declarator
                name: (identifier) @name
                type: (type_annotation) @type
            )
        )
        (lexical_declaration
            (variable_declarator
                name: (identifier) @name
                type: (type_annotation) @type
            )
        )
        (export_statement
            declaration: (lexical_declaration
                (variable_declarator
                    name: (identifier) @name
                    type: (type_annotation) @type
                )
            )
        )
    ]
)
