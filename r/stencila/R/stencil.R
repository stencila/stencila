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
Stencil <- function(content) {
    stencil <- new("Stencil")
    if(!missing(content)) stencil$content(content)
    return(stencil)
}

#' Load content into a stencil
#'
#' @name Stencil-content
#' @aliases content,Stencil-method
#' @export
#' 
#' @examples
#' # Create a stencil...
#' stencil <- Stencil()
#' # ... and set the HTML content of this stencil
#' stencil$content("<p>Hello world!</p>")
#' # ... or, equivalently
#' content(stencil,"<p>Hello world!</p>")
setGeneric("content",function(object,content) standardGeneric("content"))
setMethod("content",c("Stencil","ANY"),function(object,content) object$content(content))

Stencil_content <- function(stencil,content){
  if(missing(content)) return(call_('Stencil_content_get',stencil@pointer))
  else call_('Stencil_content_set',stencil@pointer,content)
}

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
    return(stencil$content())
})

# Stencil creation using code

# A sink for sending HTML nodes
# sink_ is an environment so these things can be refered to by reference (see http://www.stat.berkeley.edu/~paciorek/computingTips/Pointers_passing_reference_.html)
sink_ <- new.env()
sink_$stencil <- NULL
sink_$stack <- list()
sink_$id <- 0

#' Set a stencil as the destination (the 'sink') for markup code
#'
#' @name Stencil-sink
#' @aliases sink,Stencil-method
#' @export
#' 
#' @examples
#' # Create a stencil...
#' stencil <- Stencil()
#' # ... specify it as the sink for Stencila R "markcode"
#' stencil$sink()
#' # ... or, equivalently
#' sink(stencil)
setGeneric("sink",function(stencil) standardGeneric("sink"))
setMethod("sink",c("Stencil"),function(stencil) stencil$sink())

Stencil_sink <- function(stencil){
  sink_$stencil <- stencil
  sink_$stack <- list()
  sink_$id <- 0
  NULL
}

#' Unset a stencil as the destination (the 'sink')
#'
#' @name Stencil-unsink
#' @aliases unsink,Stencil-method
#' @export
#' 
setGeneric("unsink",function(stencil) standardGeneric("unsink"))
setMethod("unsink",c("Stencil"),function(stencil) stencil$unsink())

Stencil_unsink <- function(stencil){
  #' @todo Check that this stencil is currently the sink
  if(length(sink_$stack)>0){
    # Convert the stack of HTML nodes into a HTMl string
    connection <-  textConnection("html_string", "w",local=T)
    stream <- function(...) cat(...,sep='',file=connection)
    for(index in 1:length(sink_$stack)) html(sink_$stack[[index]],stream)
    close(connection)
    html_string <- paste(html_string,collapse='\n')
    # Load html into the stencil
    print(html_string)
    #stencil$load(html_string)
  }
  # This stencil is no longer the sink...
  sink_$stencil <- NULL
  # Clear the stack and reset ids ...
  sink_$stack <- list()
  sink_$id <- 0
}

# A HTML node class
Node <- setClass(
  'Node' 
)

TextNode <- setClass(
  'TextNode',
  representation = representation(
    text = 'character'
  ),
  contains = 'Node'
)

ElementNode <- setClass(
  'ElementNode',
  representation = representation(
    id = 'character',
    name = 'character',
    attributes = 'list',
    children = 'list'
  ),
  contains = 'Node'
)

#' Convert a HTML node into a HTML string
#'
#' @name Stencil-html
#' @aliases html,Node,function-method html,Node,missing-method
#' @export
#' 
setGeneric('html',function(node,stream) standardGeneric('html'))

setMethod('html',c('TextNode','function'),function(node,stream){
  stream(node@text)
})
  
setMethod('html',c('ElementNode','function'),function(node,stream){
  stream('<',node@name)
  if(length(node@attributes)>0){
    for(index in 1:length(node@attributes)){
      name <- names(node@attributes)[index]
      value <- node@attributes[[index]]
      stream(' ',name,'="',value,'"')
    }
  }
  stream('>')
  
  for(child in node@children) stream(html(child,stream))
  
  stream('</',node@name,'>')
})

setMethod('html',c('ElementNode','missing'),function(node,stream){
  connection <-  textConnection("html_string", "w",local=T)
  stream <- function(...) cat(...,sep='',file=connection)
  
  html(node,stream)
  
  close(connection)
  paste(html_string,collapse='\n')
})

# Function for creating a node from arbitrary arguments
node <- function(name,...){
  # Increment the global id and create a new Element
  sink_$id <- sink_$id + 1
  node <- ElementNode(
    id = paste(name,sink_$id,sep="#"),
    name = name
  )
  # Grab arguments and iterate over them
  args <- list(...)
  if(length(args)>0){
    for(index in 1:length(args)){
      # Extract name and value
      # If none of the args are names then names(args) is NULL
      name <- ifelse(is.null(names(args)),'',names(args)[index])
      value <- args[[index]]
      if (inherits(value,'ElementNode')){
        # A node is added as a child, even if it is named (the name is ignored)
        node@children <- c(node@children,value)
        # Pop the child node off the stack
        sink_$stack[[value@id]] <- NULL
      }
      else if(nchar(name)>0){
        # A named argument is made into an attribute
        attribute = list()
        attribute[[name]] = as.character(value)
        node@attributes = c(node@attributes,attribute)
      } else {
        # Anything else is added as a test node child 
        node@children = c(node@children,TextNode(text=as.character(value)))
      }
    }
  }
  # Add this node to the stack
  sink_$stack[[node@id]] <- node
  # Return this node
  invisible(node)
}

#' Create a HTML paragraph <p>
#'
#' @export
p <- function(...) node('p',...)

#' Create a HTML document division <div>
#'
#' @export
div <- function(...) node('div',...)

#' Create a HTML anchor <a>
#'
#' @export
a <- function(...) node('a',...)
