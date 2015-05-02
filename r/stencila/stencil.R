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
        contexts = function(value) get_(.self,'Stencil_contexts_get'),

        # Getter/setter for stencil context
        context = function(value){
            if(missing(value)) .context
            else attach(value)
        }
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

        html_get = function(indent=true){
            method_(.self,'Stencil_html_options',indent)
        },

        import = function(path) method_(.self,'Stencil_import',path),
        export = function(path) method_(.self,'Stencil_export',path),
        read = function(path="") method_(.self,'Stencil_read',path),
        write = function(path="") method_(.self,'Stencil_write',path),

        # Currently, rather than invoking multiple inheritance (ie.e a stecnil derived from a Html::Node)
        # implement these methods for Stencils and HtmlNodes
        select = function(selector) method_(.self,'Stencil_select',selector),

        attach = function(context){
            if(!is.null(context)) detach()
            if(inherits(context,'Context')) .context <<- context
            else .context <<- Context(context)
            # Assign this stencil to the 'self' context
            # variable so it can be accessed from there
            assign('self',.self,envir=.context$top())
            method_(.self,'Stencil_attach',context)
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
            url <- method_(.self,'Stencil_serve')
            if(wait){
                cat(url,'\n')
                Sys.sleep(wait)
            }
        },
        view = function(){
            if(is.null(.context)) attach(Context())
            method_(.self,'Stencil_view')
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
        },
        compile = function(){
            attach(Context())
            method_(.self,'Stencil_compile')
        }
    )
)
