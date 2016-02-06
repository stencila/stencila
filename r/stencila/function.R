#' @include component.R
NULL

#' The Function class
#'
#' Use this function to create a function!
#'
#' @export
#' @name Function
Function <- function(initializer=NULL) {
    new('Function',initializer)
}
setRefClass(
    'Function',
    contains = 'Component',
    fields = list(
        # A 'private' field for holding the actual R function
        .function = 'ANY'
    ),
    methods = list(
        initialize = function(initializer=NULL){
            callSuper()
            if(!is.null(initializer)) {
                load(initializer)
            }
        },
        load = function(content,format='native'){
            if(format=='native' | format=='name'){
                if(format=='native'){
                    # Content should be a function
                    if(mode(content)!='function'){
                      stop(paste0('Content argument is expected to be of mode function, got mode: ',mode(content)))
                    }
                } else {
                    # Content should be a sting
                    if(mode(content)!='character'){
                      stop(paste0('Content argument is expected to be of mode character got mode: ',mode(content)))
                    }
                }

                # Set the internal function field
                .function <<- if(format=='native') content else get(content)

                # Use the Rd based documentation, if any, for the function
                # Get the right help file and convert it into a list
                # The "deparse(substitute(..))" converts the expression into a character 
                # and is what help does internally if it is not provided with a character string
                # anyway.
                # Thanks to Jeroen at http://stackoverflow.com/questions/8918753/r-help-page-as-object
                name <- if(format=='native') deparse(substitute(content)) else content
                rd_files <- help(name)
                if (length(rd_files)>0) {
                    # Currently, taking the first found file
                    rd <- utils:::.getHelpFile(rd_files[1])
                    names(rd) <- substring(sapply(rd, attr, "Rd_tag"),2)

                    temp_args <- rd$arguments
                    rd$arguments <- NULL
                    docs <- lapply(rd, unlist)
                    docs <- lapply(docs, paste, collapse="")

                    temp_args <- temp_args[sapply(temp_args , attr, "Rd_tag") == "\\item"]
                    temp_args <- lapply(temp_args, lapply, paste, collapse="")
                    temp_args <- lapply(temp_args, "names<-", c("arg", "description"))
                    docs$arguments <- temp_args
                } else {
                  docs <- list()
                }

                # Convert the documentation list into a list for the C++ function
                extract <- function(name){
                    value <- docs[[name]]
                    if(!is.null(value)) value
                    else ''
                }
                rd_list <- list()
                rd_list$name <- name
                rd_list$title <- extract('title')
                rd_list$summary <- extract('description')
                rd_list$details <- extract('details')
                rd_list$examples <- extract('examples')
                rd_list$see <- extract('seealso')
                rd_list$references <- extract('references')
                rd_list$parameters <- docs$arguments
                rd_list$return <- extract('value')
                method_(.self,'Function_rd_set',rd_list)
            } else {
                method_(.self,'Function_load',content,format)
            }
        },
        json = function(content){
            get_set_(.self,'Function_json',content)
        },
        call = function(...){
            .function(...)
        }
    )
)
