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
    contains = 'Component',
    methods = list(
        initialize = function(initialiser=NULL,...){
            callSuper(...)
            if(!is.null(initialiser)) call_('Theme_initialise',.pointer,toString(initialiser))
        },
        style = function(value){
            get_set_(.self,'Theme_style_get','Theme_style_set',value)
        },
        behaviour = function(value){
            get_set_(.self,'Theme_behaviour_get','Theme_behaviour_set',value)
        },
        compile = function(){
            get_set_(.self,'Theme_compile')
        }
    )
)
