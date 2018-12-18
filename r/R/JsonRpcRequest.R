#' @export
JsonRpcRequest <- R6::R6Class("JsonRpcRequest",
  public = list(
    jsonrpc = NULL,
    id = NULL,
    method = NULL,
    params = NULL,

    initialize = function(id=NULL, method=NULL, params=NULL) {
        self$jsonrpc <- "2.0"
        self$id <- id
        self$method <- method
        self$params <- params
    }
  )
)
