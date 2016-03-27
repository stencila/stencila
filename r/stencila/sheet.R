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
            # Attach a spread
            attach(Spread())
            # Initialise (includes read)
            if(!is.null(initialiser)){
                call_('Sheet_initialise',.pointer,toString(initialiser))
            }
        },

        load = function(string,format='tsv'){
            method_(.self,'Sheet_load',string,format)
        },
        dump = function(format='tsv'){
            method_(.self,'Sheet_dump',format)
        },

        import = function(path, at="A1"){
            method_(.self,'Sheet_import',path,at)
        },
        export = function(path){
            method_(.self,'Sheet_export',path)
        },

        graphviz = function(path=''){
            method_(.self,'Sheet_graphviz',path)
        },

        read = function(path=""){
            method_(.self,'Sheet_read',path)
        },
        write = function(path=""){
            method_(.self,'Sheet_write',path)
        },

        store = function(){
            method_(.self,'Sheet_store')
        },
        restore = function(){
            method_(.self,'Sheet_restore')
        },

        page = function(path){
            if(missing(path)) method_(.self,'Sheet_page_get')
            else method_(.self,'Sheet_page',path)
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

        test = function(){
            method_(.self,'Sheet_test')
        },

        list = function(){
            method_(.self,'Sheet_list')
        },
        content = function(id){
            method_(.self,'Sheet_content',id)
        }
    )
)
