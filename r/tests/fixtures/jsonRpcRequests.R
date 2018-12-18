# Example requests
requests <- list(

  # Import a 100k Markdown document
  import_text = JsonRpcRequest$new(
    id = 1,
    method = "import",
    params = list(
      thing = sample(LETTERS, 100000, TRUE),
      format = "text/markdown"
    )
  ),

  # Execute a simple code cell
  execute_cell_simple = JsonRpcRequest$new(
    id = 2,
    method = "execute",
    params = list(
      thing = list(
        type = "Cell",
        programmingLanguage = "R",
        text = "x + y",
        inputs = list(
          list(
            type = "Var",
            name = "x",
            value = 6
          ),
          list(
            type = "Var",
            name = "y",
            value = 7
          )
        )
      )
    )
  ),

  # Execute a call with a data table arg
  execute_call = JsonRpcRequest$new(
    id = 3,
    method = "execute",
    params = list(
      thing = list(
        type = "Call",
        func = "http://example.org/functions/filter",
        args = list(
          list(
            type = "Datatable",
            columns = list(
                list(
                    type = "Column",
                    name = "col1",
                    data = list(
                        type = "Array",
                        values = runif(10000)
                    )
                ),
                list(
                    type = "Column",
                    name = "col2",
                    data = list(
                        type = "Array",
                        values = sample(LETTERS, 10000, TRUE)
                    )
                )
            )
          )
        )
      )
    )
  )
)
