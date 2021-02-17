#' Stencila for R.
#'
#' @name stencila
#' @docType package
#' @useDynLib libstencila_r, "wrap__init", "wrap__serve"
NULL

#' Call Rust function `init()`
#'
#' @export
init <- function(manifest) {
  .Call(wrap__init, as.character(substitute(manifest)))
}

#' Call Rust function `serve()`
#'
#' @export
serve <- function() {
  .Call(wrap__serve)
}

