#' @include stencila.R
NULL

#' @export
signin = function(token){
	if(missing(token)){
		username <- readline("Username: ")
		cat("Password: ")
		system("stty -echo")
		password <- readline()
		system("stty echo")
		call_('hub_signin_pass',username,password)
	}
	else if(token=='*'){
		call_('hub_signin_envvar')
	}
	else {
		call_('hub_signin_token',token)
	}
}

#' @export
username = function(){
	call_('hub_username')
}

#' @export
signout = function(){
	call_('hub_signout')
}
