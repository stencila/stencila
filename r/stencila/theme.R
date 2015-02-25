#' @include component.R
NULL

#' The Theme class
#'
#' Use this function to create a theme
#'
#' @export
#' @name Theme
Theme <- function(initialiser=NULL) {
    new('Theme',initialiser)
}
setRefClass(
    'Theme',
    fields = list(
    ),
    contains = 'Component',
    methods = list(
    )
)
