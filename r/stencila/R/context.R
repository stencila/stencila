#' @include shortcuts.R
NULL

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
    
    if(missing(envir)) envir <- new.env(parent=baseenv())
    else if(inherits(envir,'environment')) envir <- envir
    else if(is.list(envir)) envir <- list2env(envir,parent=baseenv())
    else if(is.atomic(envir) & inherits(envir,'character')){
        if(envir==".") envir <- parent.frame()
        else stop(paste('unrecognised environment flag:',envir))
    }
    else stop(paste('unrecognised environment class:',paste(class(envir),collapse=",")))
    self$stack <- list(envir)
    
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
    
    ##################################
    
    self$get <- function(expression) {
        return(eval(parse(text=expression),envir=self$top()))
    }
    
    self$set <- function(name,expression){
        env <- self$top()
        value <- eval(parse(text=expression),envir=env)
        assign(name,value,envir=env)
        return(self)
    }
    
    ##################################
    # "execute" method
    # 
    # Executes some code
    self$execute <- function(code){
        eval(parse(text=code),envir=self$top())
        return(self)
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
            if(inherits(expr,'result')){
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
    # "image" elements
    # 
    # Creates a new graphics device then executes the code
    # (which is expected to write to the device) and then 
    # returns the additional nodes.
    self$image_begin <- function(type){
        if(type!='svg') stop(paste('Image type not supported:',type))
        # Create a temporary filename
        filename = tempfile()
        # Create an SVG graphics device
        svg(filename)
        # Store that filename for later
        assign('_svg_',filename,envir=self$top())
    }
    
    self$image_end  <- function(){
        # Get filename
        filename <- base::get('_svg_',envir=self$top())
        # Close all graphics devices. In case the
        # code opened new ones, we really need to closethem all
        graphics.off()
        # Determine file size so we know how many bytes to read in
        bytes = file.info(filename)$size
        # Read in the SVG file and return it
        svg = readChar(filename,nchars=bytes)
        return(svg)
    }
    
    ##################################
    # "if" elements
    # 
    # Returns a boolean evaluation of expression
    
    self$test <- function(expression){
        value <- self$get(expression)
        return(as.logical(value))
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
        return(iterator$more())
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
            return(TRUE)
        } else {
            #No more items so exit the loop
            self$exit()
            return(FALSE)
        }
    }
    
    return(self)
}
