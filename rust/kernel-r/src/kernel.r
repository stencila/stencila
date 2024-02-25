#!/usr/bin/env R

# During development, set DEV environment variable to True
DEV_MODE = Sys.getenv("DEV") == "true"

# Define constants based on development status
READY = ifelse(DEV_MODE, "READY", "\U0010ACDC")
LINE = ifelse(DEV_MODE, "|", "\U0010ABBA")
EXEC = ifelse(DEV_MODE, "EXEC", "\U0010B522")
EVAL = ifelse(DEV_MODE, "EVAL", "\U001010CC")
FORK = ifelse(DEV_MODE, "FORK", "\U0010DE70")
INFO = ifelse(DEV_MODE, "INFO", "\U0010EE15")
PKGS = ifelse(DEV_MODE, "PKGS", "\U0010BEC4")
LIST = ifelse(DEV_MODE, "LIST", "\U0010C155")
GET = ifelse(DEV_MODE, "GET", "\U0010A51A")
SET = ifelse(DEV_MODE, "SET", "\U00107070")
REMOVE = ifelse(DEV_MODE, "REMOVE", "\U0010C41C")
END = ifelse(DEV_MODE, "END", "\U0010CB40")

# Ensure that required packages are attached and installed
requires <- function() {
  # On Mac and Linux require `parallel` package for forking
  # This is a base package (see `rownames(installed.packages(priority="base"))`)
  # so we don't try to install it as with other packages
  if (.Platform$OS.type == "unix") library(parallel)

  pkgs <- c("jsonlite", "base64enc")

  install <- NULL
  for (pkg in pkgs) {
    if (!suppressWarnings(require(pkg, character.only = TRUE, quietly = TRUE))) {
      install <- c(install, pkg)
    }
  }

  if (length(install) > 0) {
    # Ensure that the user has a place that they can install packages
    # Note that `R_LIBS_USER` is set to a default value at R startup (if not already set)
    lib <- Sys.getenv("R_LIBS_USER")
    dir.create(lib, recursive = TRUE, showWarnings = FALSE)
    # Add the lib to lib paths for any other installs in this session
    .libPaths(lib)
    for (pkg in install) {
      install.packages(pkg, quiet = TRUE, repo = "https://cloud.r-project.org/")
      require(pkg, character.only = TRUE, quietly = TRUE)
    }
  }
}

# All of `invisible`, `capture.output` and `suppressMessages` are required
# here to keep the call to `requires()` truly quiet on all platforms
invisible(capture.output(suppressMessages(requires())))

# Get stdio streams
stdin <- file("stdin", "r")
stdout <- stdout()
stderr <- stderr()

# Functions to print an `ExecutionMessage` to stderr
message <- function(msg, level, error_type = NULL) {
  message <- list(
    type = "ExecutionMessage",
    level = level,
    message = msg,
    errorType = error_type
  )

  stack_trace <- paste(capture.output(traceback()), collapse = "\n")
  if (!startsWith(stack_trace, "No traceback available")) {
    message$stackTrace <- stack_trace
  }

  write(
    paste0(
      toJSON(
        message,
        auto_unbox = TRUE,
        force = TRUE,
        null = "null",
        na = "null"
      ),
      END
    ),
    stderr
  )
}
info <- function(msg) message(msg, "Info")
warning <- function(msg) message(msg, "Warn")
error <- function(error, error_type = "RuntimeError") message(error$message, "Error", error_type)
interrupt <- function(condition, error_type = "Interrupt") message("Code execution was interrupted", "Error", error_type)

# Serialize an R object as JSON
to_json <- function(value) {
  if (inherits(value, "recordedplot") || inherits(value, "ggplot")) {
    toJSON(plot_to_image_object(value))
  } else if (inherits(value, "table")) {
    # The functions `summary` and `table` return class "table" results
    # Currently, just "print" them. In the future, we may convert these to Datatables.
    toJSON(unbox(paste(utils::capture.output(base::print(value)), collapse = "\n")))
  } else if (is.data.frame(value)) {
    toJSON(dataframe_to_datatable(value), na = "null")
  } else if (
    is.null(value) ||
    is.logical(value) ||
    is.numeric(value) ||
    is.character(value) ||
    is.matrix(value) ||
    is.array(value) ||
    is.vector(value) ||
    is.list(value)
  ) {
    # Return value because, for these types, `toJSON()` will convert
    # to the appropriate JSON type e.g. a matrix to an array of arrays
    toJSON(
      value,
      auto_unbox = TRUE,
      force = TRUE,
      null = "null",
      na = "null"
    )
  } else {
    toJSON(unbox(paste(utils::capture.output(base::print(value)), collapse = "\n")))
  }
}

# Deserialize an R object from JSON
from_json <- function(json) {
  value <- try(fromJSON(json, simplifyVector = FALSE), silent = TRUE)

  if (inherits(value, "try-error")) {
    json # Fallback to deserializing as string
  } else if (is.list(value) && length(value$type) != 0) {
    switch(value$type,
      Datatable = dataframe_from_datatable,
      identity
    )(value)
  } else {
    value
  }
}

# Create a Stencila ArrayHint from an R vector
vector_to_array_hint <- function(values) {
  list(
    type = "ArrayHint",
    length = unbox(length(values)),
    item_types = switch(class(values),
      logical = list(unbox("Boolean")),
      numeric = list(unbox("Number")),
      double = list(unbox("Number")),
      factor = list(unbox("String")),
      character = list(unbox("String")),
      default = NULL
    ),
    minimum = unbox(min(values)),
    maximum = unbox(max(values)),
    nulls = unbox(sum(is.na(values)))
  )
}

# Create a Stencila DatatableHint for an R data.frame
dataframe_to_datatable_hint <- function(df) {
  row_names <- attr(df, "row.names")
  if (!identical(row_names, seq_len(nrow(df)))) {
    columns <- list(vector_to_datatable_column_hint("name", row_names))
  } else {
    columns <- NULL
  }

  columns <- c(columns, Filter(function(column) !is.null(column), lapply(colnames(df), function(colname) {
    vector_to_datatable_column_hint(colname, df[[colname]])
  })))

  list(
    type = unbox("DatatableHint"),
    rows = unbox(nrow(df)),
    columns = columns
  )
}

# Create a Stencila DatatableColumnHint from an R vector representing a data.frame column
vector_to_datatable_column_hint <- function(name, values) {
  list(
    type = unbox("DatatableColumnHint"),
    name = unbox(name),
    item_type = switch(class(values),
      logical = unbox("Boolean"),
      integer = unbox("Integer"),
      numeric = unbox("Number"),
      double = unbox("Number"),
      factor = unbox("String"),
      character = unbox("String"),
      default = NULL
    ),
    minimum = if (is.numeric(values) || is.character(values)) { unbox(min(values, na.rm = TRUE)) } else { NULL },
    maximum = if (is.numeric(values) || is.character(values)) { unbox(max(values, na.rm = TRUE)) } else { NULL },
    nulls = unbox(sum(is.na(values)))
  )
}


# Create a Stencila Datatable from an R data.frame
dataframe_to_datatable <- function(df) {
  row_names <- attr(df, "row.names")
  if (!identical(row_names, seq_len(nrow(df)))) {
    columns <- list(vector_to_datatable_column("name", row_names))
  } else {
    columns <- NULL
  }

  columns <- c(columns, Filter(function(column) !is.null(column), lapply(colnames(df), function(colname) {
    vector_to_datatable_column(colname, df[[colname]])
  })))
  
  list(
    type = unbox("Datatable"),
    columns = columns
  )
}

# Create a Stencila DatatableColumn from an R vector representing a data.frame column
#
# Because a factor's levels are always a character vector, factors are converted into a
# column with `validator.items` of type `EnumValidator` with `values` containing the levels.
vector_to_datatable_column <- function(name, values) {
  if (is.factor(values)) {
    validator <- list(type = unbox("EnumValidator"), values = levels(values))
    values <- as.character.factor(values)
  } else if (is.logical(values)) {
    validator <- list(type = unbox("BooleanValidator"))
  } else if (is.numeric(values)) {
    validator <- list(type = unbox("NumberValidator"))
  } else if (is.character(values)) {
    validator <- list(type = unbox("StringValidator"))
  } else {
    validator <- NULL
  }

  list(
    type = unbox("DatatableColumn"),
    name = unbox(name),
    validator = list(type = unbox("ArrayValidator"), itemsValidator = validator),
    values = values
  )
}

# Create an R data.frame from a Stencila Datatable
dataframe_from_datatable <- function(dt) {
  columns = list()
  for(column in dt$columns) {
    name <- column$name
    validator <- column$validator
    values <- column$values
    if (validator$type == "BooleanValidator") {
      columns[[name]] <- as.logical(values)
    } else if (validator$type == "IntegerValidator") {
      columns[[name]] <- as.integer(values)
    } else if (validator$type == "NumberValidator") {
      columns[[name]] <- as.number(values)
    } else if (validator$type == "StringValidator") {
      columns[[name]] <- as.character(values)
    } else if (validator$type == "EnumValidator") {
      columns[[name]] <- as.factor(values, levels = validator$values)
    } else {
      columns[[name]] <- values
    }
  }
  as.data.frame(columns)
}

# Convert a R plot to an `ImageObject`
plot_to_image_object <- function(value, options = list(), format = "png") {
  # Create a new graphics device for the format, with a temporary path.
  # The tempdir check is needed when forking.
  filename <- tempfile(fileext = paste0(".", format), tmpdir = tempdir(check=TRUE))
  width <- try(as.numeric(options$width))
  height <- try(as.numeric(options$height))

  func <- get(format)
  func(
    filename,
    width = ifelse(is.numeric(width) && length(width) == 1, width, 10),
    height = ifelse(is.numeric(height) && length(width) == 1, height, 10),
    units = "cm",
    res = 150
  )
  base::print(value)
  grDevices::dev.off()

  list(
    type = unbox("ImageObject"),
    contentUrl = unbox(paste0("data:image/", format, ";base64,", base64encode(filename)))
  )
}

# Monkey patch `print` to encode individual objects
print <- function(x, ...) write(paste0(to_json(x), END), stdout)

# Expose `unbox` so that users can, for example, show a single number vector as a number
unbox <- jsonlite::unbox

# Create environment in which code will be executed
envir <- new.env()

# Execute lines of code
execute <- function(lines) {
  code <- paste0(lines, collapse = "\n")
  compiled <- tryCatch(parse(text = code), error = identity)
  if (inherits(compiled, "simpleError")) {
    error(compiled, "SyntaxError")
  } else {
    # Set a default graphics device to avoid window popping up or a `Rplot.pdf`
    # polluting local directory. 
    # `CairoPNG` is preferred instead of `png` to avoid "a forked child should not open a graphics device"
    # which arises because X11 can not be used in a forked environment.
    # The tempdir `check` is needed when forking.
    file <- tempfile(tmpdir = tempdir(check=TRUE))
    tryCatch(
      Cairo::CairoPNG(file),
      error = function(cond) png(file, type = "cairo")
    )
    # Device control must be enabled for recordPlot() to work
    dev.control("enable")

    # Execute the code
    value <- tryCatch(
      eval(compiled, envir, .GlobalEnv),
      message = info,
      warning = warning,
      error = error,
      interrupt = interrupt
    )

    # Ignore any values that are not visible
    if (!withVisible(value)$visible) {
      value <- NULL
    }

    # Capture plot and clear device
    rec_plot <- recordPlot()
    if (!is.null(rec_plot[[1]])) {
      value <- rec_plot
    }
    dev.off()  

    if (!is.null(value)) {
      # Only return value if last line is not blank, a comment, or an assignment
      last <- tail(lines, 1)
      blank <- nchar(trimws(last)) == 0
      comment <- startsWith(last, "#")
      assignment <- grepl("^\\s*\\w+\\s*(<-|=)\\s*", last)
      if (!blank && !comment && !assignment) print(value)
    }
  }
}

# Evaluate an expression
evaluate <- function(expression) {
  compiled <- tryCatch(parse(text = expression), error = identity)
  if (inherits(compiled, "simpleError")) {
    error(compiled, "SyntaxError")
  } else {
    value <- tryCatch(
      eval(compiled, envir, .GlobalEnv),
      message = info,
      warning = warning,
      error = error,
      interrupt = interrupt
    )

    print(value)
  }
}

# Get runtime information
get_info <- function() {
  print(list(
    type = unbox("SoftwareApplication"),
    name = unbox("R"),
    softwareVersion = unbox(paste(version$major, version$minor, sep = ".")),
    operatingSystem = unbox(paste(version$os, version$arch))
  ))
}

# Get a list of packages available
get_packages <- function() {
  pkgs <- installed.packages()
  for (name in rownames(pkgs)) {
    print(list(
      type = unbox("SoftwareSourceCode"),
      programmingLanguage = unbox("R"),
      name = unbox(name),
      version = unbox(pkgs[name, "Version"])
    ))
  }
}

# List variables in the context
list_variables <- function() {
  vars <- ls(envir = envir)
  for (name in vars) {
    value <- get(name, envir = envir)
    native_type <- class(value)
    result <- node_type_and_hint(value)

    print(list(
      type = unbox("Variable"),
      programmingLanguage = unbox("R"),
      name = unbox(name),
      nativeType = unbox(native_type),
      nodeType = unbox(result$node_type),
      hint = result$hint
    ))
  }
}

# Get the node type and a hint for an R value
node_type_and_hint <- function(value) {
  if (length(value) == 0 && is.null(value)) {
    list(node_type = "Null")
  } else if (length(value) == 1) {
    if (is.logical(value)) {
      list(node_type = "Boolean", hint = unbox(value))
    } else if (is.integer(value)) {
      list(node_type = "Integer", hint = unbox(value))
    } else if (is.numeric(value) || is.double(value)) {
      list(node_type = "Number", hint = unbox(value))
    } else if (is.character(value)) {
      list(node_type = "String", hint = list(
        type = unbox("StringHint"),
        chars = unbox(nchar(value))
      ))
    } else {
      list(node_type = NULL, hint = NULL)
    }
  } else if (is.data.frame(value)) {
    list(node_type = "Datatable", hint = dataframe_to_datatable_hint(value))
  } else if (is.list(value)) {
    if (length(value$type) == 1) {
      list(node_type = value$type)
    } else {
      keys <- names(value)
      if (is.null(keys)) {
        list(node_type = "Array", hint = list(
          type = unbox("ArrayHint"),
          length = unbox(length(value))
        ))
      } else {
        values <- lapply(value, function(item) node_type_and_hint(item)$hint)
        names(values) <- NULL
        list(node_type = "Object", hint = list(
          type = unbox("ObjectHint"),
          length = unbox(length(value)),
          keys = keys,
          values = values
        ))
      }
    }
  } else {
    list(node_type = "Array", hint = vector_to_array_hint(value))
  }
}

# Get a variable
get_variable <- function(name) {
  value <- try(get(name, envir = envir), silent = TRUE)
  if (!inherits(value, "try-error")) {
    print(value)
  }
}

# Set a variable
set_variable <- function(name, value) {
  assign(name, from_json(value), envir = envir)
}

# Remove a variable
remove_variable <- function(name) {
  remove(list = name, envir = envir)
}

# Fork the kernel instance
# The `eval_safe` function of https://github.com/jeroen/unix provides an alternative 
# implementation of fork-exec for R. We might use it in the future.
fork <- function(pipes) {
  # The Rust process will kill the child so use `estranged` to avoid zombie processes
  # (because this process still has an entry for the child)
  process <- parallel:::mcfork(estranged = TRUE)
  if (!inherits(process, "masterProcess")) {
    # Parent process: return pid of the fork
    print(unbox(process$pid))
  } else {
    # Close file descriptors so that we're not interfering with
    # parent's file descriptors
    closeAllConnections()
    stdin <<- file(pipes[1], open = "r", raw = TRUE)
    stdout <<- file(pipes[2], open = "w", raw = TRUE)
    stderr <<- file(pipes[3], open = "w", raw = TRUE)
  }
}

# Indicate that ready
write(READY, stdout)
write(READY, stderr)

task <<- NULL
saved_task <<- NULL
while (!is.null(stdin)) {
  tryCatch({
    # A SIGINT does not interrupt `readLines` but instead gets fired just after it when the next
    # line is read. So, we have to save the task in case interrupt was inadvertently called during
    # readline and then "replay" it on the next loop.
    task <<- readLines(stdin, n = 1) #ifelse(is.null(saved_task), readLines(stdin, n = 1), saved_task)
    saved_task <<- NULL

    # If there is no task from `readLines` it means `stdin` was closed, so exit gracefully
    if (length(task) == 0) quit(save = "no")

    lines <- strsplit(task, LINE, fixed = TRUE)[[1]]
    task_type <- lines[1]

    # Ignore if, for some reason, line was empty
    if (is.na(task_type)) {
      return
    }

    if (task_type == EXEC) {
      if (length(lines) > 1) execute(lines[2:length(lines)])
    }
    else if (task_type == EVAL) evaluate(lines[2])
    else if (task_type == INFO) get_info()
    else if (task_type == PKGS) get_packages()
    else if (task_type == LIST) list_variables()
    else if (task_type == GET) get_variable(lines[2])
    else if (task_type == SET) set_variable(lines[2], lines[3])
    else if (task_type == REMOVE) remove_variable(lines[2])
    else if (task_type == FORK) fork(lines[2:length(lines)])
    else error(list(message = paste("Unrecognized task:", task_type), error_type = "MicrokernelError"))
  },
  message = info,
  warning = warning,
  error = error,
  interrupt = function(condition) {
    if (DEV_MODE) quit(save = "no")
    else saved_task <<- task
  })

  write(READY, stdout)
  flush(stdout)
  write(READY, stderr)
  flush(stderr)
}
