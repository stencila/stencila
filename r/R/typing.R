# Functions for type definitions and checking
# This implements functions for runtime checking that are
# not available natively in R.

#' Any type
#' 
#' @return A `list` of class `Any`
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
#' @return A `list` of class `Array` describing the valid `items` of an array
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
#' @return A `list` of class `Union` describing the valid sub `types` of a union type
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
#'
#' @param ... The values in the enumeration
#' @return A `list` of class `Enum` describing the valid `values` in an enumeration
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
#'
#' @param obj The object to get the last class for
last_class <- function(obj) {
  utils::tail(class(obj), n = 1)
}

#' Is a value of a particular class
#'
#' @param value The value to check
#' @param clas The class the check against
is_class <- function(value, clas) {
  last_class(value) == clas
}

#' Does a value conform to the type?
#'
#' @param value The value to check
#' @param type The type to check against
is_type <- function(value, type) { # nolint
  type_class <- last_class(type)
  if (type_class == "function") {
    # Capture the function name and call this function with that
    func_name <- deparse(substitute(type))
    is_type(value, func_name)
  } else if (type_class == "character") {
    if (type == "NULL") return(is.null(value))
    else if (type == "numeric" && typeof(value) == "integer") return(TRUE)
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
#'
#' @param node The node to declare as a scalar
as_scalar <- function(node) {
  # Make other values "scalar" so that they are "unboxed"
  # when serialized to JSON
  class(node) <- c("scalar", class(node))
  node
}

#' Coerce a value to conform to the type
#' Principally, marks values as scalar where possible
#'
#' @param value The value to coerce
#' @param type The type to coerce it to
as_type <- function(value, type) { #nolint
  primitive_types <- c("logical", "numeric", "character")

  # Make singular primitive types scalar
  if (
    is.character(type) && type %in% primitive_types &&
    length(value) == 1 && mode(value) %in% primitive_types
  ) {
    return(as_scalar(value))
  }
  else if (is_class(type, "Array")) {
    # Flatten lists of primitives to vectors of primitives
    if (
      is.character(type$items) && type$items %in% primitive_types &&
      is.list(value)
    ) {
      return(unlist(value))
    }
    # Make singular primitives within lists scalar
    else if (
      is_class(type$items, "Any") ||
      is_class(type$items, "Union") && any(match(type$items$types, primitive_types))
    ) {
      scalarize <- function(item) {
        if (length(item) == 1 && mode(item) %in% primitive_types) as_scalar(item)
        else item
      }
      if (is.list(value)) return(lapply(value, scalarize))
      else if (is.vector(value)) return(sapply(value, scalarize, USE.NAMES = FALSE))
    }
  }
  return(value)
}

#' Check that a value is present if required and conforms to the
#' specified type for a property.
#'
#' @param type_name The name of the type that they property is on
#' @param property_name The name of the property
#' @param is_required Is a value for the property required?
#' @param is_missing Is a value for the property missing?
#' @param value The value to check
#' @param type The type to check against
check_property <- function(type_name, property_name, is_required, is_missing, type, value) {
  if (is_required && is_missing) {
    stop(
      paste0(type_name, "$", property_name, " is required"),
      call. = FALSE
    )
  }

  if (is_missing) return()

  # Convert functions to function names before passing to is_type
  if (is.function(type)) type <- deparse(substitute(type))

  value <- as_type(value, type)
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
