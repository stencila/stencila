#' The Extension class
#'
#' @name Extension
#' @export
Extension <- function() {
    new('Extension')
}
setRefClass(
	'Extension',
	fields = list(
		# A 'private' field which holds the pointer to the C++-side
		# representation of this instance
		.pointer = 'externalptr'
	),
	methods = list(
		initialize = function(pointer=NULL,...){
			callSuper(...)
			if(is.null(pointer)) pointer <- call_(paste0(class(.self)[1],'_new'))
			.pointer <<- pointer
		}
	)
)

# Convenience functions for working with Stencila R `Extension`es.
# To avoid polluting the method list of classes these are not declared as 
# methods

# Call a C++ function in the Stencila R extension module
call_ <- function(symbol,...){
    .Call(symbol,...,PACKAGE=dll_name)
}
# Call a method and wrap the result appropriately
method_ <- function(instance,symbol,...){
    # Do the call
    result <- call_(symbol,instance$.pointer,...)
    # If the return is NULL then return instance
    # so method chaining can be used...
    if(is.null(result)) return(invisible(instance))
    # Otherwise return the result after wrapping it
    else {
        # Convert an externalptr object into an instance of a Stencila R class
        if(typeof(result)=='externalptr'){
            class <- call_('Stencila_class',result)
            result <- new(class,pointer=result)
            return(result)
        }
        return(result)
    }
}
# Call a getter function; raise an error if an attempt
# is made to set the field
get_ <- function(instance,symbol,value){
    if(missing(value)) method_(instance,symbol)
    else stop('Read only field')
}
# Call getter or setter function as appropriate
get_set_ <- function(instance,get_symbol,set_symbol,value){
    if(missing(value)) method_(instance,get_symbol)
    else method_(instance,set_symbol,value)
}
