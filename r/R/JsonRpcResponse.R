#' @export
JsonRpcResponse <- R6::R6Class("JsonRpcResponse",
  public = list(
    jsonrpc = NULL,
    id = NULL,
    result = NULL,
    error = NULL,

    initialize = function(id=NULL, result=NULL, error=NULL) {
        self$jsonrpc <- "2.0"
        self$id <- id
        self$result <- result
        self$error <- error
    }
  )
)
