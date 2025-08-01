#!/usr/bin/env R

# During development, set DEV environment variable to True
DEV_MODE = Sys.getenv("DEV") == "true"

# Define constants based on development status
READY = ifelse(DEV_MODE, "READY", "\U0010ACDC")
LINE = ifelse(DEV_MODE, "|", "\U0010ABBA")
EXEC = ifelse(DEV_MODE, "EXEC", "\U0010B522")
EVAL = ifelse(DEV_MODE, "EVAL", "\U001010CC")
FORK = ifelse(DEV_MODE, "FORK", "\U0010DE70")
BOX = ifelse(DEV_MODE, "BOX", "\U0010B0C5")
INFO = ifelse(DEV_MODE, "INFO", "\U0010EE15")
PKGS = ifelse(DEV_MODE, "PKGS", "\U0010BEC4")
LIST = ifelse(DEV_MODE, "LIST", "\U0010C155")
GET = ifelse(DEV_MODE, "GET", "\U0010A51A")
SET = ifelse(DEV_MODE, "SET", "\U00107070")
REMOVE = ifelse(DEV_MODE, "REMOVE", "\U0010C41C")
END = ifelse(DEV_MODE, "END", "\U0010CB40")

# Set UTF-8 locale if necessary to ensure above codes are output properly
current_locale <- Sys.getlocale("LC_CTYPE")
if (!grepl("UTF-8|utf8", current_locale, ignore.case = TRUE)) {
  tryCatch(
    Sys.setlocale(category = "LC_ALL", locale = "C.UTF-8"),
    warning = function(w) print(paste("Warning:", w$message)),
    error = function(e) {
      tryCatch(
        Sys.setlocale(category = "LC_ALL", locale = "en_US.UTF-8"),
        warning = function(w) print(paste("Warning:", w$message)),
        error = function(e) {
          print(paste("Error:", e$message))
          print("Failed to set UTF-8 locale")
        }
      )
    }
  )
}

# Ensure that required packages are attached and installed
requires <- function() {
  pkgs <- c("jsonlite", "base64enc")

  if (.Platform$OS.type == "unix") {
    # On Mac and Linux...

    # Attach `parallel` package for forking
    # This is a base package (see `rownames(installed.packages(priority="base"))`)
    # so we don't try to install it as with other packages
    library(parallel)

    # Require Cairo to support creating new graphics devices in forks
    pkgs <- c(pkgs, "Cairo")
  }

  # Determine which packages need to be installed
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
      install.packages(pkg, quiet = TRUE, repos = "https://cloud.r-project.org/")
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
  if (inherits(msg, 'condition')) {
    msg <- conditionMessage(msg)
  } else {
    msg <- trimws(as.character(ifelse(is.list(msg) && "message" %in% names(msg), msg$message, msg)))
  }

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
warning <- function(msg) message(msg, "Warning")
exception <- function(error, error_type = "RuntimeError") message(error, "Exception", error_type)
interrupt <- function(condition, error_type = "Interrupt") message("Code execution was interrupted", "Exception", error_type)

# Serialize an R object as JSON
to_json <- function(value, ...) {
  if (inherits(value, "recordedplot") || inherits(value, "ggplot")) {
    toJSON(plot_to_image_object(value, ...))
  } else if (is_leaflet(value)) {
    toJSON(leaflet_to_image_object(value))
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

  columns <- c(columns, Filter(function(column)!is.null(column), lapply(colnames(df), function(colname) {
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

  columns <- c(columns, Filter(function(column)!is.null(column), lapply(colnames(df), function(colname) {
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
  for (column in dt$columns) {
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
plot_to_image_object <- function(value, width = 480, height = 480) {
  # Create a new graphics device for the format, with a temporary path.
  # The tempdir check is needed when forking.
  filename <- tempfile(fileext = paste0(".png"), tmpdir = tempdir(check = TRUE))
  png(
    filename,
    width = if (!is.na(width)) width else getOption('repr.plot.width', 480),
    height = if (!is.na(height)) height else getOption('repr.plot.height', 480),
  )
  base::print(value)
  grDevices::dev.off()

  list(
    type = unbox("ImageObject"),
    contentUrl = unbox(paste0("data:image/png;base64,", base64encode(filename)))
  )
}

# Check if a value is a Leaflet map (htmlwidget)
is_leaflet <- function(value) {
  inherits(value, "leaflet") && inherits(value, "htmlwidget")
}

# Convert a Leaflet map to an `ImageObject` with HTML content
leaflet_to_image_object <- function(value) {
  if (requireNamespace("htmlwidgets", quietly = TRUE)) {
    # Use htmlwidgets to render the widget as HTML
    temp_file <- tempfile(fileext = ".html", tmpdir = tempdir(check = TRUE))
    htmlwidgets::saveWidget(value, temp_file, selfcontained = TRUE)
    html_content <- paste(readLines(temp_file, warn = FALSE), collapse = "\n")
    unlink(temp_file)
    
    list(
      type = unbox("ImageObject"),
      mediaType = unbox("text/html"),
      contentUrl = unbox(html_content)
    )
  } else {
    # Fallback if htmlwidgets is not available
    list(
      type = unbox("ImageObject"),
      mediaType = unbox("text/plain"),
      contentUrl = unbox("Leaflet map (htmlwidgets package required for rendering)")
    )
  }
}

# Monkey patch `print` to encode individual objects excepting those designed to
# generate content for interpolation into documents
print <- function(x, ...) {
  if (inherits(x, 'xtable')) {
    base::print(x, ...)
  } else {
    write(paste0(to_json(x, ...), END), stdout)
  }
}

# Expose `unbox` so that users can, for example, show a single number vector as a number
unbox <- jsonlite::unbox

# Create environment in which code will be executed
envir <- new.env()

# Extract an option from Knitr style attribute comments
# https://quarto.org/docs/reference/cells/cells-knitr.html
extract_option <- function(lines, aliases) {
  pattern <- sprintf(
    "^\\s*#\\|\\s*(?:%s)\\s*:\\s*([0-9]+(?:\\.[0-9]+)?)\\s*$",
    paste(aliases, collapse = "|")
  )

  m <- regexec(pattern, lines, perl = TRUE, ignore.case = TRUE)
  captures <- regmatches(lines, m)

  vals <- vapply(captures, function(x) {
    if (length(x) == 2) as.numeric(x[2]) else NA
  }, numeric(1))

  vals[!is.na(vals)][1]
}

# Execute lines of code
#
# An alternative to most of the code in this function would be to use the
# `evaluate` package as follows. Currently, we don't do this to avoid
# adding another dependency to the microkernel.
#
#    evaluate::evaluate(
#      code,
#      envir,
#      .GlobalEnv,
#      output_handler = new_output_handler(
#        text = print,
#        graphics = print,
#        message = info,
#        warning = warning,
#        error = exception,
#        value = print
#      )
#    )
#
execute <- function(lines) {
  # Detect any graphics device settings in Knitr style comments
  # Currently assume to be in inches
  current_plot_width <- NA
  current_plot_height <- NA
  if (any(grepl("^\\s*#\\|", lines, perl = TRUE))) {
    current_plot_width <- extract_option(lines, c("fig\\.width", "fig-width", "width")) * 72
    current_plot_height <- extract_option(lines, c("fig\\.height", "fig-height", "height")) * 72
  }

  code <- paste0(lines, collapse = "\n")
  compiled <- tryCatch(parse(text = code), error = identity)
  if (inherits(compiled, "simpleError")) {
    exception(compiled, "SyntaxError")
  } else {
    # Set a default graphics device to avoid window popping up or a `Rplot.pdf`
    # polluting local directory. 
    # `CairoPNG` is preferred instead of `png` to avoid "a forked child should not open a graphics device"
    # which arises because X11 can not be used in a forked environment.
    # The tempdir `check` is needed when forking.
    file <- tempfile(tmpdir = tempdir(check = TRUE))
    tryCatch(
      Cairo::CairoPNG(file),
      error = function(cond) png(file, type = "cairo")
    )
    # Device control must be enabled for recordPlot() to work
    dev.control("enable")

    value_and_visible <- NULL
    for (expr in compiled) {
      value_and_visible <- withCallingHandlers(
        withVisible(eval(expr, envir, .GlobalEnv)),
        warning = function(msg) {
          warning(msg)
          invokeRestart("muffleWarning")
        },
        interrupt = interrupt
      )
    }

    # If the last value was a ggplot, explictly print it (withCallingHandlers will not do that
    # implicitly like tryCatch does). This is where ggplot emits warnings associated with a plot
    if (inherits(value_and_visible$value, "ggplot")) {
      withCallingHandlers(
        base::print(value_and_visible$value),
        warning = function(msg) {
          warning(msg)
          invokeRestart("muffleWarning")
        },
        interrupt = interrupt
      )
    }

    # Capture plot and clear device
    rec_plot <- recordPlot()
    if (!is.null(rec_plot[[1]])) {
      value_and_visible$value <- rec_plot
      value_and_visible$visible <- TRUE
    }
    dev.off()

    # Ignore any values that are not visible
    if (!is.null(value_and_visible) && value_and_visible$visible) {
      print(value_and_visible$value, width = current_plot_width, height = current_plot_height)
    }
  }
}

# Evaluate an expression
evaluate <- function(expression) {
  compiled <- tryCatch(parse(text = expression), error = identity)
  if (inherits(compiled, "simpleError")) {
    exception(compiled, "SyntaxError")
  } else {
    value <- withCallingHandlers(
      eval(compiled, envir, .GlobalEnv),
      warning = function(msg) {
        message(msg, "Warning")
        invokeRestart("muffleWarning")
      },
      error = exception,
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
    url = unbox(file.path(R.home("bin"), "R")),
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

    # Remove the current random seed. It will be regenerated the next
    # time a random number is generated. If this is not done, each fork
    # will have the same random seed as the parent kernel, which leads
    # to unexpected non-randomness when running forks
    rm(.Random.seed, envir = globalenv())
  }
}

# Restrict the capabilities of the kernel
#
# - erases potentially secret environment variables
# - restricts filesystem writes
# - restricts process management
# - restricts network access
#
# Instead on fully monkey patching, this just overrides functions in the kernel's environment.
# Monkey patching in R is not straight forward. We tried using `assignInNamespace` et al
# but found we could not unlock the base environment. 
# https://dlukes.github.io/monkey-patching-in-r.html
box_ <- function() {
  # Remove sensitive environment variables
  envs <- Sys.getenv()
  to_remove <- grep("SECRET|KEY|TOKEN", names(envs), ignore.case = TRUE, value = TRUE)
  for (var in to_remove) {
    Sys.unsetenv(var)
  }
  
  # Restrict filesystem writes
  readonly_error <- function(...) {
    stop("Write access to filesystem is restricted in boxed kernel", call. = FALSE)
  }

  assign("file", function (description, open, ...) {
    if (open %in% c('r', 'rt', 'rb')) {
      base::file(description, open, ...)
    } else {
      readonly_error()
    }
  }, envir = envir)

  for(name in c(
    'file.append',
    'file.copy',
    'file.create',
    'file.link',
    'file.remove',
    'file.rename',
    'file.symlink',
    'unlink',
    'dir.create',
    'dir.remove',
    'Sys.chmod',
    'write',
    'write.csv',
    'write.csv2',
    'write.table'
  )) {
    assign(name, readonly_error, envir = envir)
  }
  
  # Restrict process management
  process_error <- function(...) {
    stop("Process management is restricted in boxed kernel", call. = FALSE)
  }
  for(name in c(
    'system', 'system2', 'kill'
  )) {
    assign(name, process_error, envir = envir)
  }

  # Restrict network access
  network_error <- function(...) {
    stop("Network access is restricted in boxed kernel", call. = FALSE)
  }
  for(name in c(
    'socketConnection', 'url', 'download.file'
  )) {
    assign(name, network_error, envir = envir)
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
    else if (task_type == EVAL) {
      # Note: if multiple lines provided then joined with space
      if (length(lines) == 2) evaluate(lines[2])
      else if (length(lines) > 2) evaluate(paste(lines[2:length(lines)], collapse = " "))
    }
    else if (task_type == INFO) get_info()
    else if (task_type == PKGS) get_packages()
    else if (task_type == LIST) list_variables()
    else if (task_type == GET) get_variable(lines[2])
    else if (task_type == SET) set_variable(lines[2], lines[3])
    else if (task_type == REMOVE) remove_variable(lines[2])
    else if (task_type == FORK) fork(lines[2:length(lines)])
    else if (task_type == BOX) box_()
    else exception(list(message = paste("Unrecognized task:", task_type), error_type = "MicrokernelError"))
  },
  warning = warning,
  error = exception,
  interrupt = function(condition) {
    if (DEV_MODE) quit(save = "no")
    else saved_task <<- task
  })

  write(READY, stdout)
  flush(stdout)
  write(READY, stderr)
  flush(stderr)
}
