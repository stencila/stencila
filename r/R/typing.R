# Functions for type definitions and checking
# This implements functions for runtime checking that are
# not available natively in R.

#' Any type
#' @export
Any <- function() {
  self <- list()
  class(self) <- "Any"
  self
}

format.Any <- function(x) { # nolint
  "Any()"
}

print.Any <- function(x) { # nolint
  print(format(x)) # nocov
}


#' Array type
#'
#' @param items The type that items in the array should be
#' @export
Array <- function(items) {
  self <- list(
    items = if (is.function(items)) deparse(substitute(items)) else items
  )
  class(self) <- "Array"
  self
}

format.Array <- function(type) { # nolint
  paste0("Array(", format(type$items), ")")
}

print.Array <- function(x) { # nolint
  print(format(x)) # nocov
}

#' Union type
#'
#' @param ... The types in the union
#' @export
Union <- function(...) {
  args <- as.list(match.call())[-1]
  types <- lapply(args, function(arg) {
    # For functions, get the function name, otherwise return the value e.g. a Union
    value <- eval(arg)
    if (is.function(value)) as.character(arg) else value
  })
  self <- list(types = types)
  class(self) <- "Union"
  self
}

format.Union <- function(type) { # nolint
  paste0("Union(", paste(lapply(type$types, format), collapse = ", "), ")")
}

print.Union <- function(x) { # nolint
  print(format(x)) # nocov
}

#' An enumeration
#' @export
Enum <- function(...) {
  self <- list(values = c(...))
  class(self) <- "Enum"
  self
}

format.Enum <- function(type) { # nolint
  paste0("Enum(", paste(type$values, collapse = ", "), ")")
}

print.Enum <- function(x) { # nolint
  print(format(x)) # nocov
}


#' Get the last class for an object
#' The last class is usually the "highest" in the inheritance tree
last_class <- function(obj) {
  utils::tail(class(obj), n = 1)
}

#' Is a value of a particular class
is_class <- function(value, clas) {
  last_class(value) == clas
}

#' Does a value conform to the type?
is_type <- function(value, type) { # nolint
  type_class <- last_class(type)
  if (type_class == "function") {
    # Capture the function name and call this function with that
    func_name <- deparse(substitute(type))
    is_type(value, func_name)
  } else if (type_class == "character") {
    if (type == "NULL") return(is.null(value))
    else inherits(value, type)
  } else if (type_class == "Any") {
    TRUE
  } else if (type_class == "Array") {
    if (is.null(value) || inherits(value, "Entity")) {
      # Not array-like
      FALSE
    } else if (is.list(value)) {
      # Check all items in list are of type
      for (item in value) {
        if (!is_type(item, type$items)) return(FALSE)
      }
      TRUE
    } else if (is.vector(value)) {
      # Create an instance of the mode of the vector
      # and check that it is correct type
      inst <- get(mode(value))()
      is_type(inst, type$items)
    } else if (is.factor(value)) {
      # Factors are valid Array("character")
      is_type(character(), type$items)
    } else {
      FALSE
    }
  } else if (type_class == "Union") {
    for (subtype in type$types) {
      if (is_type(value, subtype)) return(TRUE)
    }
    FALSE
  } else if (type_class == "Enum") {
    mode(value) == mode(type$values) && value %in% type$values
  } else {
    FALSE
  }
}

#' Declare that a node is scalar
as_scalar <- function(node) {
  # Make other values "scalar" so that they are "unboxed"
  # when serialized to JSON
  class(node) <- c("scalar", class(node))
  node
}

#' Check that a value is present if required and conforms to the
#' specified type for a property.
check_property <- function(type_name, property_name, is_required, is_missing, type, value) {
  if (is_required && is_missing) {
    stop(
      paste0(type_name, "$", property_name, " is required"),
      call. = FALSE
    )
  }

  if (is_missing) return()

  if (is_class(type, "Array")) {
    # Flatten lists to vectors where possible
    if (is.list(value) && is.character(type$items) && type$items %in% c("logical", "numeric", "character")) {
      value <- unlist(value)
    }
  } else {
    value <- as_scalar(value)
  }

  # Convert functions to function names before passing to is_type
  if (is.function(type)) type <- deparse(substitute(type))

  if (!is_type(value, type)) {
    stop(
      paste0(
        type_name, "$", property_name, " is type ", last_class(value),
        ", expected type ", format(type)
      ),
      call. = FALSE
    )
  }

  value
}
