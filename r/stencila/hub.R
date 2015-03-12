#' @include extension.R
NULL

#' @export
signin = function(){
	username <- readline("Username: ")
	cat("Password: ")
	system("stty -echo")
	password <- readline()
	system("stty echo")
	call_('hub_signin',username,password)
}

#' @export
username = function(){
	call_('hub_username')
}

#' @export
signout = function(){
	call_('hub_signout')
}

#' @export
get = function(address){
	call_('Component_get',address)
}
