#' @export
Request <- R6::R6Class("Request",
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

#' @export
Response <- R6::R6Class("Response",
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
