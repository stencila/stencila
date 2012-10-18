#
#
# See http://digitheadslabnotebook.blogspot.co.nz/2011/06/environments-in-r.html (and links therein)
# for a useful explanation of environments

DefaultIterator <- function(target){
  self <- new.env()
  class(self) <- "DefaultIterator"
  
  self$target = target
  self$index = 0
  self$size = length(self$target)
  
  self$more <- function(){
    return(self$index<self$size)
  }
  
  self$step <- function(){
    self$index = self$index +1
    return(self$target[[self$index]])
  }
  
  self
}

iterate <- function(target){
  UseMethod('iterate')
}
iterate.default <- function(target){
  return(DefaultIterator(target))
}

#########################################

Context <- function(){

  self <- new.env()
  
  self$stack <- list(globalenv())
  
  ##################################
  
  self$push <- function(item){
  	self$stack[[length(self$stack)+1]] <- item
  }
  
  self$pop  <- function() {
    self$stack[[length(self$stack)]] <- NULL
  }
  
  self$top  <- function() {
  	return(self$stack[[length(self$stack)]])
  }
  
  ##################################
  
  self$get <- function(expression) {
    return(eval(parse(text=expression),envir=self$top()))
  }
  
  self$set <- function(name,expression){
    env = self$top()
    value <- eval(parse(text=expression),envir=env)
  	assign(name,value,envir=env)
  }
  
  ##################################
  # "with" and "block" directives
  #
  # Call enter('expression') at start of a "with" element
  # Call enter() at start of a "block" element
  # Call exit() at the end of a "with" or "block" element
  
  self$enter <- function(expression=NULL){
    parent <- self$top()
    if(is.null(expression)) env <- new.env(parent=parent)
    else env <- list2env(self$get(expression),parent=parent) #Use list2env rather than as.enviroment because it allows use to define parent
    self$push(env)
    return(env)
  }
  
  self$exit <- function(){
    self$pop()
  }
  
  ##################################
  # "each" declaration
  #
  # Call begin('item','items') at start of an "each" element
  # Call next() at end of each element
  
  self$begin <- function(item,expression){
    # Enter a new anonymous block that forms the namespace for the loop
    loop = self$enter()
    # Assign special variables into the loop namespace
    # so that when the step() method is called it knows which variables to get
    # and which to set
    assign('_item_',item,envir=loop)
    # 
    items <- eval(parse(text=expression),envir=loop)
    assign('_items_',iterate(items),envir=loop)
  }
  
  self$step <- function(){
    loop = self$top()
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
    } else {
      #No more items so exit the loop
      self$exit()
    }
  }

  return(self)
}
