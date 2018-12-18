#' @export
JsonRpcError <- R6::R6Class("JsonRpcError",
  public = list(
    code = NULL,
    message = NULL,
    data = NULL,

    initialize = function(code=NULL, message=NULL, data=NULL) {
        self$code <- code
        self$message <- message
        self$data <- data
    }
  )
)
