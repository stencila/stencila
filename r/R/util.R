Any <- function () {
  self <- list()
  class(self) <- "Any"
  self
}

format.Any <- function (type) {
  "Any()"
}

Array <- function (items) {
  self <- list(items=items)
  class(self) <- "Array"
  self
}

format.Array <- function (type) {
  paste0("Array(", paste(sapply(type$items, format), collapse=", "), ")")
}

Union <- function (...) {
  self <- list(types=as.character(c(...)))
  class(self) <- "Union"
  self
}

format.Union <- function (type) {
  paste0("Union(", paste(sapply(type$types, format), collapse=", "), ")")
}

isType <- function (value, type) {
  if(class(type) == "Any") {
    TRUE
  } else if (class(type) == "character") {
    type_obj <- get(type)
    if (class(type_obj) %in% c('Any', 'Array', 'Union')) isType(value, type_obj)
    else inherits(value, type)
  } else if (class(type) == "Array") {
    if(class(value) != "list") return(FALSE)
    for(item in value) {
      if(!isType(item, type$items)) return(FALSE)
    }
    TRUE
  } else if(class(type) == "Union") {
    inherits(value, type$types)
  } else {
    FALSE
  }
}

assertType <- function (value, type) {
  if(!isType(value, type)) stop(paste("value is type", class(value), "not expected type", format(type)), call. = FALSE)
  value
}

setProp <- function (node, name, type, value) {
  if(!isType(value, type)) stop(paste("value for", name, "is type", class(value), "not expected type", format(type)), call. = FALSE)
  node[[name]] <- value
}
