responses <- list(
    cell = JsonRpcResponse$new(
        id = 1,
        result = list(
            type = "Cell"
        )
    ),
    error = JsonRpcResponse$new(
        id = 1,
        error = list(
            message = "An error happened"
        )
    )
)
