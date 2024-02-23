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
  write(
    paste0(
      toJSON(
        list(
          type = "ExecutionMessage",
          level = level,
          message = msg,
          error_type = error_type
        ),
        auto_unbox = T,
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

# Monkey patch `print` to encode individual objects
print <- function(x, ...) write(paste0(toJSON(x, auto_unbox = T), END), stdout)

# Expose `unbox` so that users can, for example, show a single number vector as a number
unbox <- jsonlite::unbox

# Execute lines of code
execute <- function(lines) { }

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
    if (!is.null(value)) {
      print(value)
    }
  }
}

# Get runtime information
get_info <- function() { }

# Get a list of packages available
get_packages <- function() { }

# List variables in the context
list_variables <- function() { }

# Get a variable
get_variable <- function(name) { }

# Set a variable
set_variable <- function(name, value) { }

# Remove a variable
remove_variable <- function(name) { }

# Fork the kernel instance
fork <- function(pipes) { }

# Create environment in which code will be executed
envir <- new.env()

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
    task <<- ifelse(is.null(saved_task), readLines(stdin, n = 1), saved_task)

    # If there is no task from `readLines` it means `stdin` was closed, so exit gracefully
    if (length(task) == 0) quit(save = "no")

    lines <- strsplit(task, LINE, fixed = TRUE)[[1]]

    task_type <- lines[1]
    switch(task_type,
      EXEC = function() execute(lines[2:length(lines)]),
      EVAL = function() evaluate(lines[2:length(lines)]),
      INFO = get_info,
      PKGS = get_packages,
      LIST = list_variables,
      GET = function() get_variable(lines[2]),
      SET = function() set_variable(lines[2], lines[3]),
      REMOVE = function() remove_variable(lines[2]),
      FORK = function() fork(lines[2], lines[3]),
      function() error(list(message = paste("Unrecognized task:", task_type), error_type = "MicrokernelError"))
    )()

    saved_task <<- NULL

    write(READY, stdout)
    write(READY, stderr)
  },
  message = info,
  warning = warning,
  error = error,
  interrupt = function(condition) {
    saved_task <<- task
  })
}
