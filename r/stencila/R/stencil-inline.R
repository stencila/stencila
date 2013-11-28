# This file includes implementation of in code HTML functions
# See Stencil$target and Stencil$untarget
# Only the HTML functions at the end of this file are exported by the package

# A HTML node class
Node <- setClass(
  'Node' 
)
# A method for converting a HTML node to a HTML string
setGeneric('html_',function(node,stream) standardGeneric('html_'))

# A HTML text node
TextNode <- setClass(
  'TextNode',
  representation = representation(
    text = 'character'
  ),
  contains = 'Node'
)
# HTML node to a HTML string. Called with a function to stream the HTML to
setMethod('html_',c('TextNode','function'),function(node,stream){
  stream(node@text)
})

# A HTML element node
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
# HTML node to a HTML string. Called with a function to stream the HTML to
setMethod('html_',c('ElementNode','function'),function(node,stream){
  stream('<',node@name)
  if(length(node@attributes)>0){
    for(index in 1:length(node@attributes)){
      name <- names(node@attributes)[index]
      value <- node@attributes[[index]]
      stream(' ',name,'="',value,'"')
    }
  }
  stream('>')
  
  for(child in node@children) stream(html_(child,stream))
  
  stream('</',node@name,'>')
})
# HTML node to a HTML string. Called with no stream function; return a string. 
# Useful for debugging to convert a node into a string
setMethod('html_',c('ElementNode','missing'),function(node,stream){
  connection <-  textConnection("html_string", "w",local=T)
  stream <- function(...) cat(...,sep='',file=connection)
  
  html_(node,stream)
  
  close(connection)
  paste(html_string,collapse='\n')
})

# A sink for sending HTML nodes when they are created
# sink_ is an environment so these things can be refered to by reference 
# (see http://www.stat.berkeley.edu/~paciorek/computingTips/Pointers_passing_reference_.html)
sink_ <- new.env()
sink_$stack <- list()
sink_$id <- 0

# Make a stencil the target
sink_start_ <- function(stencil){
  sink_$stack <- list()
  sink_$id <- 0
}

# Collect up the nodes and return HTML string
sink_finish_ <- function(stencil){
  html_string <- ""
  if(length(sink_$stack)>0){
    # Convert the stack of HTML nodes into a HTML string
    connection <-  textConnection("html_string", "w",local=T)
    stream <- function(...) cat(...,sep='',file=connection)
    for(index in 1:length(sink_$stack)) html_(sink_$stack[[index]],stream)
    close(connection)
    html_string <- paste(html_string,collapse='\n')
  }
  # Clear the stack and reset ids ...
  sink_start_()
  # Return the string
  return(html_string)
}

# Function for creating a node from arbitrary arguments
# This is a 'base' function for the individual element functions
node_ <- function(name,...){
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
p <- function(...) node_('p',...)

#' Create a HTML document division <div>
#'
#' @export
div <- function(...) node_('div',...)

#' Create a HTML anchor <a>
#'
#' @export
a <- function(...) node_('a',...)
