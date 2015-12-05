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
    fields = list(
        # A 'private' field for holding an R-side `Spread` used for execution.
        # This is necessary to prevent garbage collection of the `Spread`
        # when there is a C++-side `RSpread` which holds a pointer to it.
        # A '.' prefix is used to signify this is a private field and to prevent
        # a name clash with a method name below
        .spread = 'ANY'
    ),
    methods = list(
        initialize = function(initialiser=NULL,pointer=NULL,...){
            callSuper(pointer,...)
            if(!is.null(initialiser)){
                call_('Sheet_initialise',.pointer,toString(initialiser))
            }
            attach(Spread())
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
        },

        attach = function(spread){
            .spread <<- spread
            method_(.self,'Sheet_attach',.spread)
        },
        detach = function(){
            .spread <<- NULL
            method_(.self,'Sheet_detach')
        },
        update = function(){
            method_(.self,'Sheet_update')
        },

        list = function(){
            method_(.self,'Sheet_list')
        },
        value = function(id){
            method_(.self,'Sheet_value',id)
        }
    )
)
