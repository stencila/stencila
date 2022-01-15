(module
    [
        (expression_statement (string) @comment)
        (comment) @comment
    ]
)
        
(import_statement
    name: (dotted_name) @module
)
(import_from_statement
    module_name: (dotted_name) @module
)

(call
    function: (identifier) @function (#match? @function "^open$")
    arguments: (
        argument_list
            ([(string)(identifier)] @arg)*
            ([(string)(identifier)] @arg)*
            (keyword_argument
                name: (identifier) @arg_name
                value: (string) @arg_value
            )*
            (keyword_argument
                name: (identifier) @arg_name
                value: (string) @arg_value
            )*
    )
)

(module
    (expression_statement
        (assignment
            left: (identifier) @name
            right: (_) @value
        )
    )
) 
(module
    (function_definition
      name: (identifier) @name
    )
)

((identifier) @identifer)
