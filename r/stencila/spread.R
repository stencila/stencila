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
    
    # Should this have the enclosing env as a parent?
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

    # Use method names with leading dot to disassociate them from sheet variables (e.g. in
    # use of `ls()` below)

    # Execute source in this spread's environment
    self$.execute <- function(source){
        match = regmatches(source,regexec('library\\((\\w+)\\)',source))[[1]]
        if (length(match)>0){
            package <- match[2]
            result <- tryCatch(
                library(
                    package,
                    character.only=TRUE,
                    verbose=FALSE,
                    quietly=TRUE
                ),
                error=identity
            )
            if(inherits(result,'error')){
                return(paste('error',result$message))
            } else {
                return(paste('import',package))
            }
        } else {
            return('')
        }
    }
    
    # Get the type and string representation of a value
    self$.value <- function(value){
        classes <- class(value)

        if (length(classes)==1) {
            # Conversion of type names to standard
            # names as required (default is to use native type name)
            type <- switch(classes,
                logical = 'boolean',
                numeric = 'real',
                character = 'string',
                classes
            )
        } else {
            type <- classes[1]
        }

        if(type=='boolean'){
            repr <- if(value) 'true' else 'false'
        } else if(type %in% c('integer','real','string')){
            repr <- toString(value)
        } else if (type=='image_file') {
            repr <- value
        }
        else {
            repr <- paste(capture.output(print(value)),collapse="\n")
        }

        list(
            type = type,
            repr = repr
        )
    }
    
    # Evaluate an expression
    # 
    # @param prefix A useful prefix to be added to any files that may be produced from evaluation
    # @param as_string Should the return value be a string? (set to false when used internally)
    self$.evaluate <- function(expression, prefix, as_string = TRUE){
        dir.create('out', showWarnings = FALSE)

        # Create an empty device for possible graphics produced below
        filename <- file.path("out",paste0(prefix,"_temp.png"))
        png(filename, width=500, height=500)
        device <- dev.cur()
        # Record the device state to detect any changes
        device_before <- recordPlot()

        # Evaluate expression within this environment, capturing any errors
        type <- ''
        value <- tryCatch(
            eval(
                parse(text=expression),
                envir=self
            ),
            error=identity
        )
        repr <- ''

        # ggplots need to be explicitly printed within
        # the `recordPlot()` calls above and below
        is_ggplot <- FALSE
        if('ggplot' %in% class(value)) {
            valueOrError <- tryCatch(
                print(value),
                error=identity
            )
            # If there was an error printing the ggplot
            # (e.g. plot misspecification), then use the error
            # handling code below, otherwise use image capturing code
            # below
            if(inherits(valueOrError,'error')){
                value <- valueOrError
            } else {
                is_ggplot <- TRUE
            }
        }

        # Record and close device so it is written to disk
        device_after <- recordPlot()
        dev.off(device)
        if(file.exists(filename)){
            # Remove any existing files with the prefix (but not the temp file)
            file.remove(Sys.glob(file.path("out",paste0(prefix,"-*"))))
            # Get MD5 hash to uniquely identify the image in case of changes
            hash <- tools::md5sum(filename)
            # Rename the file
            new_filename <- file.path("out",paste0(prefix,'-',hash,'.png'))
            file.rename(filename,new_filename)
            filename <- new_filename
        }

        # Determine type and string representation
        if(inherits(value,'error')){
            type <- 'error'
            repr <- value$message
            value <- NA
        } else {
            # Check if device has changed
            if(!identical(device_before,device_after) | is_ggplot){
                value <- filename
                class(value) <- 'image_file'
            }
            result <- self$.value(value)
            type <- result$type
            repr <- result$repr
        }

        if (as_string) return(paste(type,repr))
        else return(list(type=type,value=value,repr=repr))
    }
    
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
    self$.set <- function(id,expression,name=""){      
        evaluation <- self$.evaluate(
            expression,
            prefix = ifelse(nchar(name),name,id),
            as_string = FALSE
        )

        assign(id,evaluation$value,envir=self)
        if(name!=""){
            assign(name,evaluation$value,envir=self)
        }
        
        return(paste(evaluation$type,evaluation$repr))
    }


    # Get a cell value
    # 
    # Name could be a cell id e.g. F5 or and name e.g. price
    self$.get <- function(name){
        value <- tryCatch(
            get(
                name,
                envir = self
            ),
            error = identity
        )
        if(inherits(value,'error')){
            result <- "error Update required"
        } else {
            result <- self$.value(value)
        }
        return(paste(result$type,result$repr))
    }
    
    # Clear one or all cell values
    # 
    # Name could be a cell id e.g. F5 or and name e.g. price
    self$.clear <- function(id="",name=""){
      if(nchar(id)){
        remove(list=id,envir=self)
        if(nchar(name)){
            remove(list=name,envir=self)
        }
      } else {
        remove(list=ls(self),envir=self)
      }
      return("")
    }
    
    # List all variable names
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
        # Use the handy `all.names` function which does the 
        # AST generation and walking for us
        return(paste(all.names(parse(text=expression)),collapse=","))
    }

    # List functions
    # 
    # Currently only returns names which consist only of "word" (i.e alpha numerics)
    # characters (i.e. ignore operator functions like "]<-" and those including a ".")
    self$.functions <- function(){
        names <- ls(baseenv())
        names <- names[grep('^\\w+$',names,perl=T)]
        paste(names,collapse=",")
    }

    # Get a function
    self$.function <- function(name){
        func <- Function()
        func$load(name, format='name')
        func$.pointer
    }

    # Read this spread from disk
    self$.read <- function(path){
        rdata = file.path(path,'context.RData')
        if (file.exists(rdata)) {
            load(
                file = rdata,
                envir = self
            )
        }
        ''
    }

    # Write this spread to disk
    self$.write <- function(path){
        rdata = file.path(path,'context.RData')
        save(
            list = ls(self),
            file = rdata,
            envir = self
        )
        ''
    }
    
    self
}
