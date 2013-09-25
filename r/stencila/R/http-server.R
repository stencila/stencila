#' @include shortcuts.R
NULL

#' The HttpServer class
#'
#' @name HttpServer
#' @aliases HttpServer-class
#' @seealso HttpServer-start HttpServer-stop HttpServer-run
#' @exportClass HttpServer
#' @export
#'
#' @examples
#' # Create a HTTP server...
#' server <- HttpServer()
class_('HttpServer')
HttpServer <- function() new("HttpServer")
