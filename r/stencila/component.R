#' @include stencila.R
NULL

#' The Component class
#'
#' @name Component
#' @export
Component <- function() {
    new('Component')
}
setRefClass(
	'Component',
	fields = list(
		# A 'private' field which holds the pointer to the C++-side
		# representation of this instance
		.pointer = 'externalptr',
		
		path = function(value) get_set_(.self,'Component_path_get','Component_path_set',value),
		address = function(value) get_(.self,'Component_address_get'),

		origin = function(value) get_(.self,'Component_origin_get'),
		commits = function(value) get_(.self,'Component_commits_get')
	),
	methods = list(
		initialize = function(pointer=NULL,...){
			callSuper(...)
			if(is.null(pointer)) pointer <- call_(paste0(class(.self)[1],'_new'))
			.pointer <<- pointer
		},
		show = function(){
		    cat(class(.self)[1],'@',address,'\n',sep='')
		},
		commit = function(message=""){
			method_(.self,'Component_commit',toString(message))
		}
	)
)

# Convenience functions for working with Stencila R `Components`.
# To avoid polluting the method list of classes these are not declared as 
# `Component` methods

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
