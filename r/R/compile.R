#' @include util.R
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

#' Compile a `CodeChunk`
compile_code_chunk <- function(chunk) { # nolint TODO: Reduce cyclometric complexity of this function
  # For convienience, allow passing a string
  if (is.character(chunk)) {
    chunk <- CodeChunk(
      programmingLanguage = "r",
      text = chunk
    )
  }

  # Code chunk "source" properties
  language <- chunk$programmingLanguage
  if (is.null(language)) language <- "r"
  else if (!(language %in% c("r", "R"))) return(chunk)
  text <- chunk$text

  # Code chunk properties augmented below
  # Use augmentation, rather than replacement, to allow
  # users to explicitly specify items in these properties
  # other than via tags in code (e.g. via a UI)
  imports <- chunk$imports
  declares <- chunk$declares
  assigns <- chunk$assigns
  alters <- chunk$alters
  uses <- chunk$uses
  reads <- chunk$reads

  # Parse any property tags in comments e.g. # @reads data.csv
  lines <- string_split(text, "\\r?\\n")
  for (property in c("imports", "declares", "assigns", "alters", "uses", "reads")) {
    regex <- paste0("#'?\\s*@", property, "\\s+(.+)")
    for (line in lines) {
      match <- string_match(line, regex)
      if (!is.null(match)) {
        values_list <- trimws(match[2])
        if (string_right(values_list) == "." && string_right(values_list, 2) != ",.") {
          values_list <- paste(substr(values_list, 1, nchar(values_list) - 1), ".", sep = ",")
        }
        values <- string_split(values_list, "\\s*,\\s*")
        assign(property, unique(c(get(property), values)))
      }
    }
  }

  # Parse the code into an AST
  ast <- as.list(parse(text = text))

  ast_walker <- function(node, depth = 0) {
    if (is.symbol(node)) {
      name <- as.character(node)
      if (!(name %in% assigns || name %in% declares)) uses <<- unique(c(uses, name))
    } else if (is.call(node)) {
      # Resolve the function name
      func <- node[[1]]
      if (is.symbol(func)) {
        # 'Normal' function call
        func_name <- as.character(func)
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
        ast_walker(node[[3]], depth + 1)
        return()
      } else if (func_name %in% assign_func_names) {
        left <- node[[2]]
        right <- node[[3]]
        if (is.call(right) && right[[1]] == "function" && depth == 0) {
          # Assignment of a function
          params <- right[[2]]
          body <- right[[3]]
          # Treat as a declaration
          func_decl <- Function(
            name = as.character(left)
          )
          if (!is.null(params)) {
            parameters <- NULL
            params <- as.list(params)
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
          # Walk the function body
          ast_walker(body, depth + 1)
          return()
        } else if (
          func_name == "assign" ||
          is.symbol(left) && (func_name == "<<-" || depth == 0)
        ) {
          # Assignment using `<<-` operator or `assign` functin
          # are always assumed to bind to the top level
          # (even though they may not e.g. see `pos` arg of `assign`)
          assigns <<- unique(c(assigns, as.character(left)))
        } else if (
          is.call(left) && (func_name == "<<-" || depth == 0)
        ) {
          # Assignment to an existing object  e.g. a$b[1] <- NULL
          # Recurse until we find the variable that is target of alteration
          walk <- function(node) {
            left <- node[[2]]
            if (is.symbol(left)) {
              name <- as.character(left)
              if (!(name %in% assigns)) alters <<- unique(c(alters, name))
            }
            else walk(left)
          }
          walk(left)
        }
      } else if (func_name %in% import_func_names) {
        # Package import
        # Get the names of the package
        if (length(node) > 1) {
          imports <<- unique(c(imports, as.character(node[[2]])))
          # Do not walk the first argument to avoid adding
          # an unquoted package name e.g. library(dplyr) to `uses`
          if (length(node) > 2) lapply(node[3:length(node)], ast_walker, depth)
          return()
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
      } else {
        # Some other function
        # Add function to `uses` if it is is not in declared
        # or is in base environment
        if (
          nchar(func_name) > 0 &&
          !(
            func_name %in% base_func_names ||
            func_name %in% sapply(declares, function(func) func$name)
          )
        ) {
          uses <<- unique(c(uses, func_name))
        }
        # Walk arguments of function
        if (func_name %in% nse_funcs_names) {
          # Do not walk args of functions that are registered as using NSE
          # TODO: only ignore args that are registered, if any.
          return()
        }
      }
    }

    # If there are child nodes, walk over them too
    if (length(node) > 1) lapply(node[2:length(node)], ast_walker, depth)
  }
  lapply(ast, ast_walker)

  # Strip property values that are after a trailing dot
  # Users can use this to control a property's content
  strip <- function(property) {
    matched <- match(".", property)
    if (is.na(matched)) property
    else property[1: (matched - 1)]
  }

  c(
    chunk,
    filter(
      list(
        imports = strip(imports),
        declares = declares,
        assigns = strip(assigns),
        alters = strip(alters),
        uses = strip(uses),
        reads = strip(reads)
      ),
      function(property) !is.null(property)
    )
  )
}
