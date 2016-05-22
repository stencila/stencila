#' @include stencila.R
NULL

#' Convert an Excel formula into an abstract syntax tree (AST)
#' 
#' @param excel Excel cell formula to parse
#' @param mode Form of AST to return: "l" give a long form nested list (which may be more convieient for analysis)
#'             whereas the default converts some AST node types into corresponding
#'             R objects (e.g. A1 into a symbol) and is more compact
#' @return AST object
#'
#' @examples
#' str(excel_ast('SUM(A1:A10)*4'))
#' str(excel_ast('SUM(A1:A10)*4','l'))
#'
#' @export
excel_ast <- function(excel, mode='') {
    call_('excel_ast', excel, mode)
}

#' Convert an Excel formula into a R expression
#'
#' @param excel Excel cell formula to parse
#' @return A R expression
#'
#' @examples
#' excel_r('SUM(3,A1/2)')
#' eval(parse(text=excel_r('MODE(1,2,2,3)')))
#'
#' @export
excel_r <- function(excel) {
    call_('excel_r', excel)
}
