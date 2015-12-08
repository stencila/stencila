#' Create a spread for a sheet
#' 
#' Spreads are to Sheets what Contexts are to Stencils. 
#' The spread holds the the variables representing the sheet.
#' 
#' @param envir The environment for the spread. Optional.
#' @param closed Should this be a closed scope (i.e. not having a parent scope)? Optional.
#'
#' @export
Spread <- function(envir, closed=FALSE) {
    
    # Will this have the enclosing env as a parent?
    parent = if(closed) baseenv() else parent.frame()

    # Create the new environment
    if(missing(envir)){
        self <- new.env(parent=parent)
    }
    else if(inherits(envir,'environment')){
        self <- envir
    }
    else if(is.list(envir)){
        self <- list2env(envir,parent=parent)
    }
    else {
        stop(paste('unrecognised environment class:',paste(class(envir),collapse=",")))
    }

    class(self) <- "Spread"

    # Use method names with leading dot to disaccosiated them from sheet variables
    
    # Assign a expression to a variable name
    # 
    # This is the method that the sheet calls for every cell which
    # has a value. For example, if cell J7 has a value '=2*pi*B4' then:
    # 
    #   spread$.set('J7','2*pi*B4')
    #   
    # or if the cell has a contant:
    # 
    #   spread$.set('B4','89.87')
    #   
    # This allows more meaningful variable names to be 
    # assigned e.g. for a constant 
    # 
    #   radius = 89.87
    # 
    #   spread$.set('B4','89.87','radius')
    #
    #  or, for an expression e.g. 
    #  
    #   circumference = 2*pi*radius
    # 
    #   spread$.set('J7','2*pi*radius','circumference')
    #
    self$.set <- function(id,expression,alias=""){
        value <- tryCatch(
            eval(
                parse(text=expression),
                envir=self
            ),
            error=identity
        )
        if(inherits(value,'error')){
            value <- paste("error:",value$message)
        }

        assign(id,value,envir=self)
        if(alias!=""){
            assign(alias,value,envir=self)
        }
        return(toString(value))
    }


    # Get a cell value
    # 
    # Name could be a cell name e.g. F5 or and alias e.g. price
    self$.get <- function(name){
        value <- get(name,envir=self)
        stream <- textConnection("text", "w")
        cat(paste(value,collapse=", "),file=stream)
        close(stream)
        return(text)
    }
    
    # Clear one or all cell values
    # 
    # Name could be a cell name e.g. F5 or and alias e.g. price
    self$.clear <- function(name){
      if(nchar(name)){
        remove(list=name,envir=self)
      } else {
        remove(list=ls(self))
      }
      return("")
    }
    
    # List all cell names
    #
    # Most likely to be used just for testing purposes
    self$.list <- function(){
      return(paste(ls(self),collapse=','))
    }

    # List the dependencies of a cell
    # 
    # Parse a cell expression to obtain all it dependencies
    # This will include variables and functions, some of which
    # may not be in the sheet
    self$.depends <- function(expression){
        # Use the hand `all.names` function which does the 
        # AST generation and walking for us
        return(all.names(parse(text=expression)))
    }
    
    self
}
