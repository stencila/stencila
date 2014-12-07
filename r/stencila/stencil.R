#' @include component.R
NULL

#' The Stencil class
#'
#' Use this function to create a stencil, optionally
#' including any initial content.
#'
#' @param content Initial HTML content of the stencil
#'
#' @export
#' @exportClass Stencil
#' @name Stencil
#' @aliases Stencil-class
#'
#' @examples
#' # Create a Stencil...
#' stencil <- Stencil()
#' # ... which is equivalent to
#' stencil <- new("Stencil")
#' # Create a Stencil and set its content
#' stencil <- Stencil('Pi has a value of: <span data-text="pi"/>')
#' # ... which is equivalent to
#' stencil <- Stencil()
#' stencil$html('Pi has a value of: <span data-text="pi"/>')
class_('Stencil','Component')
Stencil <- function(content) {
    stencil <- new('Stencil')
    if(!missing(content)) stencil$initialise(toString(content))
    stencil
}

#' Get or set the content of a Stencil as HTML
#'
#' @export
#' @name html
#' @aliases Stencil-html,Stencil-method
#' 
#' @examples
#' # Create a stencil...
#' stencil <- Stencil()
#' # ... and set 
#' stencil$html("<p>Hello world!</p>")
#' # ... or, get it's HTML content
#' stencil$html()
NULL

attr_('Stencil','html',toString)
setGeneric('html',function(instance,value) standardGeneric('html'))
setMethod('html','Stencil',Stencil_html)

attr_('Stencil','cila',toString)

# Function used below to ensure that a stencil has a context attached
Stencil_context_ensure_ <- function(stencil,context=NULL){
    if(is.null(context)){
        if(stencil$context()=="none") context <- Context(parent.frame())
    }
    else {
        if(!('Context' %in% class(context))) context <- Context(context)
    }
    call_('Stencil_attach',stencil@pointer,context)
}

#' Render a stencil object or a stencil string 
#'
#' This is a convienience function for creating, rendering and then
#' returning its content as HTML.
#' It is useful for quickly executing these three common tasks in stencil usage.
#'
#' @export
#' @name render
#' @aliases Stencil-method render,ANY-method
NULL

Stencil_render <- function(stencil,context=NULL){
    Stencil_context_ensure_(stencil,context)
    return(call_('Stencil_render',stencil@pointer))
}
setGeneric("render",function(stencil,context) standardGeneric("render"))
setMethod("render",c("ANY","ANY"),function(stencil,context){
    if(!('Stencil' %in% class(stencil))){
        stencil <- Stencil(paste("html://",stencil))
    }
    stencil$render(context)
    return(stencil$html())
})

Stencil_serve <- function(stencil,context=NULL){
    Stencil_context_ensure_(stencil,context)
    return(call_('Stencil_serve',stencil@pointer))
}

Stencil_view <- function(stencil,context=NULL){
    Stencil_context_ensure_(stencil,context)
    return(call_('Stencil_view',stencil@pointer))
}
