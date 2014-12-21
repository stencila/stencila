#R"(#"

# Load default packages as usually happens in the startup of R
# this loads important packages like stats, utils and graphics
# See http://stat.ethz.ch/R-manual/R-patched/library/base/html/Startup.html
.First.sys()

# Prevent R from printing out error messages
options(show.error.messages = FALSE)

# Set a very wide console so R does not do line wrapping of output
# 10000 is the maximum value allowed see ?options
options(width=10000)

#' Create a stencil rendering context
#' 
#' Stencils are rendered within a context. 
#' The context determines the variables that are available to the stencil.
#' Often, stencils will be rendered within the context of the R global environment.
#' However, if you want to create a different context then use this function
#'
#' See http://digitheadslabnotebook.blogspot.co.nz/2011/06/environments-in-r.html (and links therein)
#' for a useful explanation of environments
#' 
#' @param envir The environment for the context. Optional.
#'
#' @export
Context <- function(envir){
    
    self <- new.env()
    class(self) <- "Context"
    
    if(missing(envir)) envir <- parent.frame()
    else if(inherits(envir,'environment')) envir <- envir
    else if(is.list(envir)) envir <- list2env(envir,parent=baseenv())
    else stop(paste('unrecognised environment class:',paste(class(envir),collapse=",")))
    self$stack <- list(envir)
        
    ####################################################################
    # Internal convienience methods used below
    
    self$push <- function(item){
        self$stack[[length(self$stack)+1]] <- item
        return(self)
    }
    
    self$pop  <- function() {
        self$stack[[length(self$stack)]] <- NULL
        return(self)
    }
    
    self$bottom  <- function() {
        return(self$stack[[1]])
    }
    
    self$top  <- function() {
        return(self$stack[[length(self$stack)]])
    }
    
    self$get <- function(expression) {
        self$evaluate(expression)
    }
    
    self$set <- function(name,expression){
        env <- self$top()
        value <- eval(parse(text=expression),envir=env)
        assign(name,value,envir=env)
        return(self)
    }

    self$evaluate <- function(expression){
        # Evaluate an expression in the top of the stack
        result <- tryCatch(eval(parse(text=expression),envir=self$top()),error=function(error)error)
        if(inherits(result,'error')){
            stop(result$message,call.=F)
        } else {
            return(result)
        }
    }
    
    ####################################################################
    # Methods that implement the context interface
    # See the documentation for the `Context` C++ base class methods

    self$execute <- function(code,id,format="",width="",height="",units=""){
        if(format!=""){
            if(format %in% c('png','svg')){
                filename = paste0(id,'.',format)
                # Default image sizes are defined in `stencil-render.cpp` so that they
                # are consistent across contexts. Don't be tempted to replace missing values
                # with defaults here!
                width = if(width=='') stop('no width specified') else as.numeric(width)
                height = if(height=='') stop('no height specified') else as.numeric(height)
                if(units=='') stop('no units specified')
                if(format=='png'){
                    # The `res` argument must be specified unless `units='in'`
                    # Use 150ppi instead of the default 72
                    png(filename=filename,width=width,height=height,units=units,res=150)
                }
                else if(format=='svg'){
                    # The svg function does not have a units argument and assumes inches. 
                    # Absolute size does matter because it affects the relative size of the text
                    # For cm, adjust accordingly...
                    if(units=='cm'){
                        width = width/2.54
                        height = height/2.54
                    }
                    # For pixels, use a nominal resolution of 150ppi...
                    else if(units=='px'){
                        width = width/150
                        height = height/150
                    }
                    svg(filename=filename,width=width,height=height)
                }
            }
        }

        self$evaluate(code)

        if(format!=""){
            # Close all graphics devices. In case the
            # code opened new ones, we really need to close them all
            graphics.off()

            return(filename)
        }

        return("")
    }
        
    self$interact_code <- ""
    self$interact <- function(code,id){
        self$interact_code <- paste(self$interact_code,code,sep="")
        expr <- tryCatch(parse(text=self$interact_code),error=function(error)error)
        if(inherits(expr,'error')){
            if(grepl('unexpected end of input',expr$message)){
                # Input not yet complete
                return(paste("C",self$interact_code,sep=""))
            } else {
                # Syntax error
                self$interact_code <- ""
                return(paste("S",expr$message,sep=""))
            }
        } else {
            self$interact_code <- ""
            result <- tryCatch(eval(expr,envir=self$top()),error=function(error)error)
            if(inherits(result,'error')){
                # Runtime error
                return(paste("E",result$message,sep=""))
            } else {
                # Check to see if a device has been written to
                if(length(dev.list())>0){
                    # Copy to a PNG file
                    filename <- paste0(id,'.png')
                    dev.copy(png,filename=filename)
                    # Close all graphics devices
                    graphics.off()
                    return(paste("I",filename,sep=""))
                } else {
                    # show() and capture.output() actually return vectors of strings for each line
                    # so they need to be collapsed...
                    string <- paste(capture.output(show(result)),collapse='\n')
                    return(paste("R",string,sep=""))
                }
            }
        }
    }

    self$assign <- function(name,expression){
        self$set(name,expression)
    }

    self$input <- function(name,type,value){
        # Convert the string value to the appropriate R type
        # Note that for text type there is no conversion, the text value is
        # simply assigned to the variable
        # For a full list of input types see
        #   https://developer.mozilla.org/en-US/docs/Web/HTML/Element/Input
        if(type=='number') value <- as.numeric(value)
        else if(type=='date') value <- strptime(value,"%Y-%m-%d")
        else if(type=='datetime') value <- strptime(value,"%Y-%m-%d %H:%M:%S")
        # Now assign the variable
        assign(name,value,envir=self$top())
    }

    self$write <- function(expression){
        value <- self$get(expression)
        stream <- textConnection("text", "w")
        cat(paste(value,collapse=", "),file=stream)
        close(stream)
        return(text)
    }

    self$test <- function(expression){
        value <- self$get(expression)
        if(as.logical(value)) "1" else "0"
    }

    self$mark <- function(expression){
        value <- self$get(expression)
        assign('_subject_',value,envir=self$top())
        return(self)
    }
    
    self$match <- function(expression){
        value <- self$get(expression)
        subject <- base::get('_subject_',envir=self$top())
        return(value==subject)
    }

    self$unmark <- function(){
        assign('_subject_',NULL,envir=self$top())
        return(self)
    }
    
    self$enter <- function(expression=''){
        parent <- self$top()
        if(nchar(expression)==0) env <- new.env(parent=parent)
        else env <- list2env(self$get(expression),parent=parent) #Use list2env rather than as.enviroment because it allows use to define parent
        self$push(env)
        return(self)
    }
    
    self$exit <- function(){
        self$pop()
        return(self)
    }
    
    self$begin <- function(item,items){
        # Enter a new anonymous block that forms the namespace for the loop
        loop <- self$enter()$top()
        # Create an iterator for items
        items <- eval(parse(text=items),envir=loop)
        iterator <- iterate(items)
        assign('_items_',iterator,envir=loop)
        # Assign special variables into the loop namespace
        # so that when the step() method is called it knows which variables to get
        # and which to set
        assign('_item_',item,envir=loop)
        assign(item,iterator$step(),envir=loop)
        # Return flag indicating if any items
        if(iterator$more()) "1" else "0"
    }
    
    # Because "next" is an R keyword this method has to be called "next_"
    self$next_ <- function(){
        loop <- self$top()
        # Get _items_
        items <- base::get('_items_',envir=loop)
        # Go to next item
        if(items$more()){
            item <- items$step()
            assign(
                base::get('_item_',envir=loop),
                item,
                envir=loop
            )
            return("1")
        } else {
            #No more items so exit the loop
            self$exit()
            return("0")
        }
    }
    
    ####################################################################
    # Methods for read/write to disk
    
    self$read_from <- function(dir){
        base::load(paste(dir,'.RData',sep='/'),envir=self$bottom())
    }
    
    self$write_to <- function(dir){
        envir <- self$bottom()
        objs <-ls(envir)
        save(list=objs,envir=envir,file=paste(dir,'.RData',sep='/'))
    }

    return(self)
}

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
#'
#' @examples
#' i <- iterate(c(1,2,3))
#' i$step() # -> 1
#' i$step() # -> 2
#' i$more() # -> TRUE
#' i$step() # -> 3
#' i$more() # -> FALSE
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
        self$index<self$size
    }
    
    self$step <- function(){
        self$index <- self$index+1
        self$container[[self$index]]
    }
    
    self
}
iterate.default <- function(container){
    DefaultIterator(container)
}

# Iterator for data.frames. Iterates by row
DataframeIterator <- function(container){
    self <- DefaultIterator(container)
    self$size <- nrow(container)

    self$step <- function(){
        self$index <- self$index+1
        self$container[self$index,]
    }

    self
}
iterate.data.frame <- function(container){
    DataframeIterator(container)
}

# Other iterate methods will be added when
# other iterator classes are written (e.g. MatrixIterator)

#)"
