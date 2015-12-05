#' @include component.R
NULL

#' The Sheet class
#'
#' Use this function to create a sheet
#'
#' @export
#' @name Sheet
Sheet <- function(initialiser=NULL) {
    new('Sheet',initialiser)
}
setRefClass(
    'Sheet',
    contains = 'Component',
    methods = list(
        initialize = function(initialiser=NULL,pointer=NULL,...){
            callSuper(pointer,...)
            if(!is.null(initialiser)){
                call_('Sheet_initialise',.pointer,toString(initialiser))
            }
        },

        initialise = function(initialiser){
            method_(.self,'Sheet_initialise',initialiser)
        },

        load = function(string,format='tsv'){
            method_(.self,'Sheet_load',string,format)
        },
        dump = function(format='tsv'){
            method_(.self,'Sheet_dump',format)
        },

        import = function(path){
            method_(.self,'Sheet_import',path)
        },
        export = function(path){
            method_(.self,'Sheet_export',path)
        },

        read = function(path=""){
            method_(.self,'Sheet_read',path)
        },
        write = function(path=""){
            method_(.self,'Sheet_write',path)
        },

        compile = function(){
            method_(.self,'Sheet_compile')
        }
    )
)
