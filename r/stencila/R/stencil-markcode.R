# In code markup generation

# A HTML sink for sending HTMl strings of nodes to
# sink_ is an environment so these things can be refered to by reference (see http://www.stat.berkeley.edu/~paciorek/computingTips/Pointers_passing_reference_.html)
# Currently sink_$stencil is unused
sink_ <- new.env()
sink_$stencil <- NULL
sink_$connection <- NULL

#' Set a stencil as the destination (the 'sink') for markup code ('markcode')
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
    sink_$connection <- textConnection("sink_html_", "w")
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
       
   # Close the textConnection
   close(sink_$connection)
   # Obtain its text (lines are split, collapse them)
   sink_html = paste(sink_html_,collapse='\n')
   
   # Load html into the stencil
   stencil$load(sink_html)
   
   # This stencil is no longer the sink...
   sink_$stencil <- NULL
}

# A HTML node class
setClass(
    'Node',
    representation = representation(
        name = 'character',
        attributes = 'list',
        text = 'character',
        children = 'vector'
    )  
)

# Function for creating a node from arbitrary arguments
node_ <- function(name,...){
    node = new('Node',name=name)
    
    # Grab arguments and iterate over them
    args <- list(...)
    if(length(args)>0){
        for(index in 1:length(args)){
            # Extract name and value
            # If none of the args are names then names(args) is NULL
            name <- ifelse(is.null(names(args)),'',names[index])
            value <- args[[index]]
            if (inherits(value,'Node')){
                # A node is added as a child, even if it is named (the name is ignored)
                node@children = c(node@children,value)
            }
            else if(nchar(name)>0){
                # A named argument is made into an attribute
                attribute = list()
                attribute[[name]] = as.character(value)
                node@attributes = c(node@attributes,attribute)
            } else {
                # Anything else is added as a test node child 
                node@children = c(node@children,new('Node',text=as.character(value)))
            }
        }
    }
    
    node
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

#' Convert a HTML node into a HTML string
#'
#' @name Stencil-html
#' @aliases html,Node,function-method html,Node,missing-method
#' @export
#' 
setGeneric('html',function(node,stream) standardGeneric('html'))
setMethod('html',c('Node','function'),function(node,stream){
    if(length(node@text)>0){
        stream(node@text)
    } else {
        
        stream('<',node@name)
        if(length(node@attributes)>0){
            for(index in 1:length(node@attributes)){
                name <- names(node@attributes)[index]
                value <- node@attributes[[index]]
                stream(' ',name,'="',value,'" ')
            }
        }
        stream('>')
        
        for(child in node@children) stream(html(child,stream))
        
        stream('</',node@name,'>')
    }
})
setMethod('html',c('Node','missing'),function(node,stream){
    connection <-  textConnection("html_string", "w",local=T)
    stream <- function(...) cat(...,sep='',file=connection)
    
    html(node,stream)
    
    close(connection)
    paste(html_string,collapse='\n')
})

#' Send a HTML node to the current stencil sink
#'
#' @name Node-exclaimation
#' @aliases !,Node-method
#' @rdname Node-exclaimation
#' @export
#'
setMethod('!',signature(x='Node'),function(x){
    cat(
        html(x),
        sep = '',
        file = sink_$connection
    )
    invisible(NULL)
})

