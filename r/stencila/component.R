#' @include extension.R
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
    contains = 'Extension',
	fields = list(
		path = function(value) get_set_(.self,'Component_path_get','Component_path_set',value),
		address = function(value) get_(.self,'Component_address_get'),

		origin = function(value) get_(.self,'Component_origin_get'),
		commits = function(value) get_(.self,'Component_commits_get')
	),
	methods = list(
		show = function(){
		    cat(class(.self)[1],'@',address,'\n',sep='')
		},
		commit = function(message=""){
			method_(.self,'Component_commit',toString(message))
		}
	)
)
