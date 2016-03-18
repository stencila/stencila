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
        # a name clash with a method name below
        .context = 'ANY'
    ),
    contains = 'Component',
    methods = list(
        initialize = function(initialiser=NULL,pointer=NULL,...){
            callSuper(pointer,...)
            .context <<- NULL
            # Initialise from the argument
            if(!is.null(initialiser)) call_('Stencil_initialise',.pointer,toString(initialiser))
            # Attach a context, if necessary reading it from the the stencil's path
            context <- Context()
            attach(context)
        },

        initialise = function(initialiser){
            method_(.self,'Stencil_initialise',initialiser)
        },

        html = function(value){
            if(missing(value)) method_(.self,'Stencil_html_get',TRUE)
            else if(typeof(value)=='logical') method_(.self,'Stencil_html_get',value)
            else method_(.self,'Stencil_html_set',toString(value))
        },
        cila = function(value){
            get_set_(.self,'Stencil_cila_get','Stencil_cila_set',value)
        },

        title = function(){
            get_(.self,'Stencil_title_get')
        },
        description = function(){
            get_(.self,'Stencil_description_get')
        },
        keywords = function(){
            get_(.self,'Stencil_keywords_get')
        },
        authors = function(){
            get_(.self,'Stencil_authors_get')
        },
        contexts = function(){
            get_(.self,'Stencil_contexts_get')
        },

        import = function(path) method_(.self,'Stencil_import',path),
        export = function(path) method_(.self,'Stencil_export',path),
        source = function(filename){
            get_set_(.self,'Stencil_source_get','Stencil_source_set',filename)
        },
        read = function(path="") method_(.self,'Stencil_read',path),
        write = function(path="") method_(.self,'Stencil_write',path),

        restrict = function() method_(.self,'Stencil_restrict'),

        # Currently, rather than invoking multiple inheritance (i.e a stencil derived from a Html::Node)
        # implement these methods for Stencils and HtmlNodes separately
        select = function(selector) method_(.self,'Stencil_select',selector),

        context = function(value){
            if(missing(value)){
                return(.context)
            }
            else if(is.null(value)){
                # This is primarily for debugging purposes.
                # Returns the C++ string representation of the context
                return(
                    method_(.self,'Stencil_context_get')
                )
            }
            else {
                attach(value)
            }
        },
        attach = function(context){
            if(!is.null(context)) detach()
            if(inherits(context,'Context')) .context <<- context
            else .context <<- Context(context)
            method_(.self,'Stencil_attach',.context)
            # Assign this stencil to the 'self' context
            # variable so it can be accessed from there
            assign('self',.self,envir=.context$top())
        },
        detach = function(){
            .context <<- NULL
            method_(.self,'Stencil_detach')
        },
        render = function(context=NULL){
            if(!is.null(context)) attach(Context(context))
            else if(is.null(.context)) attach(Context())
            method_(.self,'Stencil_render')
        },

        serve = function(wait=0){
            if(is.null(.context)) attach(Context())
            method_(.self,'Stencil_serve')
        },
        view = function(){
            if(is.null(.context)) attach(Context())
            method_(.self,'Stencil_view')
        },
        page = function(path){
            if(missing(path)) method_(.self,'Stencil_page_get')
            else method_(.self,'Stencil_page',path)
        },

        docx = function(direction,path){
            method_(.self,'Stencil_docx',direction,path)
        },
        markdown = function(direction,path){
            method_(.self,'Stencil_markdown',direction,path)
        },
        pdf = function(direction,path){
            method_(.self,'Stencil_pdf',direction,path)
        },
        preview = function(path){
            method_(.self,'Stencil_preview',path)
        }
    )
)
