#' @include component.R
NULL

#' The Stencil class
#'
#' Use this function to create a stencil, optionally
#' including any initial content.
#'
#' @param content Initial content of the stencil
#'
#' @export
#' @name Stencil
#'
#' @examples
#' # Create a Stencil...
#' stencil <- Stencil()
#' # ... which is equivalent to
#' stencil <- new("Stencil")
#' # Create a Stencil and set its content
#' stencil <- Stencil('html://Pi has a value of: <span data-text="pi"/>')
#' # ... which is equivalent to
#' stencil <- Stencil()
#' stencil$html <- 'Pi has a value of: <span data-text="pi"/>'
Stencil <- function(initialiser=NULL) {
    new('Stencil',initialiser)
}
setRefClass(
    'Stencil',
    fields = list(
        # A 'private' field for holding an R-side `Context` used in rendering.
        # This is necessary to prevent garbage collection of the `Context`
        # when there is a C++-side `RContext` which holds a pointer to it.
        # A '.' prefix is used to signify this is a private field and to prevent
        # a name clash with method arguments
        .context = 'ANY',

        html = function(value) get_set_(.self,'Stencil_html_get','Stencil_html_set',value),
        cila = function(value) get_set_(.self,'Stencil_cila_get','Stencil_cila_set',value),

        title = function(value) get_(.self,'Stencil_title_get'),
        description = function(value) get_(.self,'Stencil_description_get'),
        keywords = function(value) get_(.self,'Stencil_keywords_get'),
        authors = function(value) get_(.self,'Stencil_authors_get'),
        contexts = function(value) get_(.self,'Stencil_contexts_get')
    ),
    contains = 'Component',
    methods = list(
        initialize = function(initialiser=NULL,...){
            callSuper(...)
            .context <<- NULL
            if(!is.null(initialiser)) call_('Stencil_initialise',.pointer,toString(initialiser))
        },

        initialise = function(initialiser){
            method_(.self,'Stencil_initialise',initialiser)
        },

        import = function(path) method_(.self,'Stencil_import',path),
        export = function(path) method_(.self,'Stencil_export',path),
        read = function(path="") method_(.self,'Stencil_read',path),
        write = function(path="") method_(.self,'Stencil_write',path),

        attach = function(context){
            if(!is.null(context)) detach()
            if(inherits(context,'Context')) .context <<- context
            else .context <<- Context(context)
            method_(.self,'Stencil_attach',context)
        },
        detach = function(){
            .context <<- NULL
            method_(.self,'Stencil_detach')
        },
        render = function(context=NULL){
            if(!is.null(context)) attach(context)
            else if(is.null(.context)) attach(Context())
            method_(.self,'Stencil_render')
        },

        serve = function(){
            if(is.null(.context)) attach(Context())
            method_(.self,'Stencil_serve')
        },
        view = function(){
            if(is.null(.context)) attach(Context())
            method_(.self,'Stencil_view')
        }
    )
)
