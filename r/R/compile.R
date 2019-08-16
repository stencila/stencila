#' @include nse_funcs.R
#' @include read_funcs.R

#' Names of function that assign
assign_func_names <- c("assign", "base::assign", "<<-", "<-", "=")

#' Names of functions that "import" packages
import_func_names <- c("library", "require", "::", ":::")

#' Names of function in the base R environment
base_func_names <- ls(baseenv())

# Temporary
Function <- function(...) list(type = "Function", ...)

compile_chunk <- function(chunk) {
  language <- chunk$language
  text <- chunk$text
  imports <- NULL
  declares <- NULL
  assigns <- NULL
  alters <- NULL
  uses <- NULL
  reads <- NULL

  # Only handle R code
  if (is.null(language) || !(language %in% c("r", "R"))) return(chunk)

  # Parse the code into an AST
  ast <- as.list(parse(text = text))

  # Record assignments that are local
  # to functions, they need to be considered
  # for `uses`, but not for `assigns`
  local_assigns <- NULL

  ast_walker <- function(node, depth = 0) {
    if (is.symbol(node)) {
      name <- as.character(node)
      if (!(name %in% assigns)) uses <<- unique(c(uses, name))
    } else if (is.call(node)) {
      # Resolve the function name
      func <- node[[1]]
      if (is.symbol(func)) {
        # 'Normal' function call
        # Add function to `uses` if it is is not in base environment
        func_name <- as.character(func)
        if (!(func_name %in% base_func_names)) {
          uses <<- unique(c(uses, func_name))
        }
      } else if (is.call(func) && func[[1]] == "::") {
        # Call of namespaced function e.g pkg::func
        # Do not add these to `uses`
        func_name <- paste0(func[[2]], "::", func[[3]])
      } else {
        # No func_name for other more complex calls
        # that do not need to be detected below e.g. instance$method()
        func_name <- ""
      }

      if (func_name == "$") {
        # Only walk the left side, not the right since they are symbols to
        # extract from an object so should not be included in `uses`
        ast_walker(node[[2]], depth)
        return()
      } else if (func_name == "function") {
        # Function definition
        # Walk the body with incremented depth
        ast_walker(node[[length(node)]], depth + 1)
        return()
      } else if (func_name %in% assign_func_names) {
        left <- node[[2]]
        right <- node[[3]]
        if (is.call(right) && right[[1]] == "function") {
          # Assignment of a function
          # Treat as a declaration
          func_decl <- Function(
            name = as.character(left)
          )
          if (!is.null(right[[2]])) {
            parameters <- NULL
            params <- as.list(right[[2]])
            print(names(params))
            for (name in names(params)) {
              param <- params[[name]]
              parameters <- c(parameters, list(
                Parameter(
                  name = name
                )
              ))
            }
            func_decl$parameters <- parameters
          }
          declares <<- c(declares, list(func_decl))

          # Only walk the function so that left is not made a `uses`
          ast_walker(right, depth)
          return()
        } else if (func == "assign") {
          # Assignment using `assign` function
          # TODO: Check the `pos` arg relative to current depth
          assigns <<- unique(c(assigns, left))
        } else if (is.symbol(left) && depth == 0) {
          # Assignment to a name
          assigns <<- unique(c(assigns, as.character(left)))
        } else if (is.call(left)) {
          # Assignment to an existing object  e.g. a$b[1] <- NULL
          # Recurse until we find the variable that is target of alteration
          walk <- function(node) {
            target <- node[[2]]
            if (is.symbol(target)) {
              if (is.null(alters) || !(target %in% alters)) {
                alters <<- c(alters, as.character(target))
              }
            } else {
              walk(target)
            }
          }
          walk(target)
        }
      } else if (func_name %in% import_func_names) {
        # Package import
        # Get the names of the package
        if (length(node) > 1) {
          imports <<- unique(c(imports, as.character(node[[2]])))
        }
      } else if (func_name %in% read_funcs_names) {
        # File read
        # Collect relevant argument(s) from function call
        args <- as.list(node[2:length(node)])
        read_func_index <- floor((match(func_name, read_funcs_names) - 1) / 2) + 1
        read_func <- read_funcs[[read_func_index]]
        if (any(read_func$names %in% names(args))) {
          files <- unlist(args[read_func$names])
        } else {
          files <- unlist(args[read_func$positions])
        }
        # Only use character arguments i.e. not symbols (variable names)
        files <- files[is.character(files)]
        if (length(files) > 0) reads <<- unique(c(reads, files))
      }
    }

    # If there are child nodes, walk over them too
    if (length(node) > 1) {
      lapply(node[2:length(node)], ast_walker, depth)
    }
  }
  lapply(ast, ast_walker)

  list(
    language = language,
    text = text,
    imports = imports,
    declares = declares,
    assigns = assigns,
    alters = alters,
    uses = uses,
    reads = reads
  )
}
