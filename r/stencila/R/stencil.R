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
#' stencil$content('Pi has a value of: <span data-text="pi"/>')
class_('Stencil')
Stencil <- function(content,language="html") {
    stencil <- new("Stencil")
    if(!missing(content)) stencil$content(content,language)
    return(stencil)
}

#' Get or set the content of a Stencil
#'
#' @name Stencil-content
#' @aliases content,Stencil-method
#' @export
#' 
#' @examples
#' # Create a stencil...
#' stencil <- Stencil()
#' # ... and set the it's content
#' stencil$content("Hello world!")
#' # ... or, equivalently
#' content(stencil,"Hello world!")
setGeneric("content",function(stencil,content,language) standardGeneric("content"))
setMethod("content",c("Stencil","ANY","character"),function(stencil,content,language) stencil$content(content,language))
setMethod("content",c("Stencil","ANY","missing"),function(stencil,content) stencil$content(content,"html"))
Stencil_content <- function(stencil,content,language){
  if(missing(language)) language <- "html"
  if(missing(content)) return(call_('Stencil_content_get',stencil@pointer,language))
  else call_('Stencil_content_set',stencil@pointer,toString(content),language)
}

#' Get or set the content of a Stencil as HTML
#'
#' @name Stencil-html
#' @aliases html,Stencil-method
#' @export
#' 
#' @examples
#' # Create a stencil...
#' stencil <- Stencil()
#' # ... and set it's HTML content
#' stencil$html("<p>Hello world!</p>")
#' # ... or, equivalently
#' html(stencil,"<p>Hello world!</p>")
setGeneric("html",function(stencil,content) standardGeneric("html"))
setMethod("html",c("Stencil","ANY"),function(stencil,content) stencil$html(content))
Stencil_html <- function(stencil,content){
  Stencil_content(stencil,content,"html")
}

#' Render a stencil object or a stencil string 
#'
#' This is a convienience function for creating, rendering and then
#' returning its content as HTML.
#' It is useful for quickly executing these three common tasks in stencil usage.
#'
#' @name Stencil-render
#' @aliases render,Stencil-method render,ANY-method
#' @export
setGeneric("render",function(stencil,context) standardGeneric("render"))
setMethod("render",c("ANY","ANY"),function(stencil,context){
    if(!('Stencil' %in% class(stencil))){
        stencil <- Stencil(stencil)
    }
    
    if(missing(context)) context <- Context(parent.frame())
    else {
        if(!('Context' %in% class(context))) context <- Context(context)
    }
    
    stencil$render(context)
    return(stencil$html())
})
