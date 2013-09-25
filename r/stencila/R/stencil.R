#' @include shortcuts.R
NULL

#' The Stencil class
#'
#' Use this function to create a stencil, optionally
#' including any initial content.
#'
#' @param content Initial HTML content of the stencil
#'
#' @name Stencil
#' @aliases Stencil-class
#' @exportClass Stencil
#' @export
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
#' stencil$load('Pi has a value of: <span data-text="pi"/>')
class_('Stencil')
Stencil <- function(content) {
    stencil <- new("Stencil")
    if(!missing(content)){
        stencil$load(content)
    }
    return(stencil)
}

#' Load content into a stencil
#'
#' @name Stencil-load
#' @aliases load,Stencil-method
#' @export
#' 
#' @examples
#' # Create a stencil...
#' stencil <- Stencil()
#' # ... and load some HTML into this stencil
#' stencil$load("<p>Hello world!</p>")
#' # ... or, equivalently
#' load(stencil,"<p>Hello world!</p>")
setGeneric("load",function(object,content) standardGeneric("load"))
setMethod("load",c("Stencil","ANY"),function(object,content) object$load(content))

#' Render a stencil object or a stencil string 
#'
#' This is a convienience function for creating, rendering and then
#' dumping a stencil.
#' It is useful for quickly executing these three common tasks in stencil usage.
#'
#' @name Stencil-render
#' @aliases render,Stencil-method render,ANY-method
#' @export
setGeneric("render",function(stencil,workspace) standardGeneric("render"))
setMethod("render",c("ANY","ANY"),function(stencil,workspace){
    if(!('Stencil' %in% class(stencil))){
        stencil <- Stencil(stencil)
    }
    
    if(missing(workspace)) workspace <- Workspace(parent.frame())
    else {
        if(!('Workspace' %in% class(workspace))) workspace <- Workspace(workspace)
    }
    
    stencil$render(workspace)
    return(stencil$dump())
})





