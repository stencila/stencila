#' Iterators

#' Create an iterator for an R object
#'
#' An iterator is a "helper" object for traversing a container object.
#' Iterators are commonly used in languages such as C++ and Java.
#' Stencila requires iterators for rendering stencils.
#' Specifically, when rendering "for" elements, iterators allow for looping over item in a container
#' using a consistent interface: one which exposes the $step() and $more() methods.
#' There is already an 'iterator' package for R.
#' We have not used that package here because Stencila requires fairly simple iterator functionality and
#' it would introduce another dependency.
#' However, users looking for more advanced and comprehensive iterators
#' should consider using the 'iterator' package.
#'
#' @param container The container object to iterate over
#' 
#' @export
iterate <- function(container){
    UseMethod('iterate')
}

# A default iterator for vectors and lists
DefaultIterator <- function(container){
    self <- new.env()
    class(self) <- "DefaultIterator"
    
    self$container <- container
    self$index <- 0
    self$size <- length(self$container)
    
    self$more <- function(){
        return(self$index<self$size)
    }
    
    self$step <- function(){
        self$index <- self$index +1
        return(self$container[[self$index]])
    }
    
    self
}
#' @export
iterate.default <- function(container){
    return(DefaultIterator(container))
}

# Other iterate methods will be added when
# other iterator classes are written (e.g. DataframeIterator, MatrixIterator)
