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
#' # ... and set the HTML content of this stencil
#' stencil$content("Hello world!")
#' # ... or, equivalently
#' content(stencil,"Hello world!")
setGeneric("content",function(stencil,content,language) standardGeneric("content"))
setMethod("content",c("Stencil","ANY","ANY"),function(stencil,content,language) stencil$content(content,language))
Stencil_content <- function(stencil,content,language){
  if(missing(language)) language = "html"
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

#' Append HTML to the content of a Stencil
#'
#' @name Stencil-html_append
#' @aliases html_append,Stencil-method
#' @export
#' 
#' @examples
#' # Create a stencil...
#' stencil <- Stencil()
#' # ... and add some HTML
#' stencil$html_append("<p>Hello world!</p>")
#' # ... or, equivalently
#' html_append(stencil,"<p>Hello world!</p>")
setGeneric("html_append",function(stencil,content) standardGeneric("html_append"))
setMethod("html_append",c("Stencil","ANY"),function(stencil,content) stencil$html_append(toString(content)))

#' Render a stencil object or a stencil string 
#'
#' This is a convienience function for creating, rendering and then
#' returning its content as HTML.
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
    return(stencil$html())
})

#' Set a stencil as the target destination for in-code HTML functions
#'
#' @name Stencil-target
#' @aliases target,Stencil-method
#' @export
#' 
#' @examples
#' # Create a stencil...
#' stencil <- Stencil()
#' # ... specify it as the target for Stencila HTML functions
#' stencil$target()
#' # ... or, equivalently
#' target(stencil)
setGeneric("target",function(stencil) standardGeneric("target"))
setMethod("target",c("Stencil"),function(stencil) stencil$target())
Stencil_target <- function(stencil){
  sink_start_(stencil)
  NULL
}

#' Unset a stencil as the target destination for in-code HTML functions
#'
#' @name Stencil-untarget
#' @aliases untarget,Stencil-method
#' @export
#' 
setGeneric("untarget",function(stencil) standardGeneric("untarget"))
setMethod("untarget",c("Stencil"),function(stencil) stencil$untarget())
Stencil_untarget <- function(stencil){
  stencil$html(sink_finish_(stencil))
  NULL
}
