#R"(#"

# Load default packages and usually happens in the startup of R
# this loads important packages like stats, utils and graphics
# See http://stat.ethz.ch/R-manual/R-patched/library/base/html/Startup.html
.First.sys()

# Prevent R from printing out error messages
options(show.error.messages = FALSE)

#' Create a stencil rendering context
#' 
#' Stencils are rendered within a context. 
#' The context determines the variables that are available to the stencil.
#' Often, stencils will be rendered within the context of the R global environment.
#' However, if you want to create a different context then use this function
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

    # An image counter for filenames
    self$images = 0
    
    ##################################
    
    self$read_from <- function(dir){
        base::load(paste(dir,'.RData',sep='/'),envir=self$bottom())
    }
    
    self$write_to <- function(dir){
        envir <- self$bottom()
        objs <-ls(envir)
        save(list=objs,envir=envir,file=paste(dir,'.RData',sep='/'))
    }
    
    ##################################
    
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
    
    ##################################
    # "execute" method

    self$execute <- function(code,format="",width="",height="",units=""){
        if(format!=""){
            if(format %in% c('png','svg')){
                self$images = self$images + 1
                filename = paste0(self$images,'.',format)
                width = if(width=="") 10 else as.numeric(width)
                height = if(height=="") 10 else as.numeric(height)
                if(units=="") units = 'cm'
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
    
    ##################################
    # "interact" method
    
    self$interact_code <- ""
    
    self$interact <- function(code){
        self$interact_code <- paste(self$interact_code,code,sep="")
        expr <- tryCatch(parse(text=self$interact_code),error=function(error)error)
        if(inherits(expr,'error')){
            if(grepl('unexpected end of input',expr$message)){
                return(paste("C",self$interact_code,sep=""))
            } else {
                self$interact_code <- ""
                return(paste("S",expr$message,sep=""))
            }
        } else {
            self$interact_code <- ""
            result <- tryCatch(eval(expr,envir=self$top()),error=function(error)error)
            if(inherits(result,'error')){
                return(paste("E",result$message,sep=""))
            } else {
                # show() and capture.output() actually return vectors of strings for each line
                # so they need to be collapsed...
                string <- paste(capture.output(show(result)),collapse='\n')
                return(paste("R",string,sep=""))
            }
        }
    }
    
    ##################################
    # "text" elements
    # 
    # Returns a text representation of the expression
    
    self$write <- function(expression){
        value <- self$get(expression)
        stream <- textConnection("text", "w")
        cat(paste(value,collapse=", "),file=stream)
        close(stream)
        return(text)
    }
    
    ##################################
    # "if" elements
    # 
    # Returns a boolean evaluation of expression
    
    self$test <- function(expression){
        value <- self$get(expression)
        if(as.logical(value)) "1" else "0"
    }
    
    ##################################
    # "switch" elements
    
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
    
    ##################################
    # "with" elements
    #
    # Call enter('expression') at start of a "with" element
    # Call enter() at start of a "block" element
    # Call exit() at the end of a "with" or "block" element
    #
    # See http://digitheadslabnotebook.blogspot.co.nz/2011/06/environments-in-r.html (and links therein)
    # for a useful explanation of environments
    
    self$enter <- function(expression=NULL){
        parent <- self$top()
        if(is.null(expression)) env <- new.env(parent=parent)
        else env <- list2env(self$get(expression),parent=parent) #Use list2env rather than as.enviroment because it allows use to define parent
        self$push(env)
        return(self)
    }
    
    self$exit <- function(){
        self$pop()
        return(self)
    }
    
    ##################################
    # "each" elements
    #
    # Call begin('item','items') at start of an "each" element
    # Call step() at end of each element
    
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
