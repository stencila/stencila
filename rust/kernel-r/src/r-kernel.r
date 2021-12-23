#!/usr/bin/env Rscript

# Ensure that required packages are installed
requires <- function () {
  pkgs <- c("jsonlite", "base64enc")
  if (.Platform$OS.type == "unix") pkgs <- c(pkgs, "parallel")

  install <- NULL
  for (pkg in pkgs) {
    if (!suppressWarnings(require(pkg, character.only = TRUE, quietly=TRUE))) {
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
# All of invisible, capture.output and suppressMessages are require here to keep this
# truely quiet on all platforms
invisible(capture.output(suppressMessages(requires())))

# Source sibling files
args <- commandArgs(trailingOnly = FALSE)
pattern <- "--file="
match <- grep(pattern, args)
file <- sub(pattern, "", args[match])
# Unescape whitespaces in file paths for macOS
dir <- gsub("\\~\\+\\~", " ", dirname(file))

source(file.path(dir, "r-codec.r"))

READY <- "\U0010ACDC"
RESULT <- "\U0010CB40"
TASK <- "\U0010ABBA"
FORK <- "\U0010DE70"

stdin <- file("stdin", "r")
stdout <- stdout()
stderr <- stderr()

# Functions to encode messages as `CodeMessage`
message <- function(msg, type) write(paste0(encode_message(msg, type), RESULT), stderr)
info <- function(msg) message(msg, "CodeInfo")
warning <- function(msg) message(msg, "CodeWarning")
error <- function(error, type = "RuntimeError") message(error$message, type)
interrupt <- function(condition, type = "CodeInterrupt") message("Code execution was interrupted", type)

# Environment in which code will be executed
envir <- new.env()

# Monkey patch `print` to encode individual objects
print <- function(x, ...) write(paste0(encode_value(x), RESULT), stdout)

# Expose `unbox` so that users can, for example show a single number vector as a number
unbox <- jsonlite::unbox

write(READY, stdout)
write(READY, stderr)

saved_task <<- NULL

while (!is.null(stdin)) {
  tryCatch(
    {
      # A SIGINT does not interrupt `readLines` but instead gets fired just after it when the next
      # line is read. So, we have to save the task in case interrupt was inadvertantly called during
      # readline and then "replay" it on the next loop.
      if (is.null(saved_task)) {
        task <<- readLines(stdin, n=1)
      } else {
        task <<- saved_task
      }
      saved_task <<- NULL

      # If there is no task from `readLines` it means `stdin` was closed, so exit gracefully
      if (length(task) == 0) quit(save = "no")

      lines <- strsplit(task, "\\n", fixed = TRUE)[[1]]

      if (lines[1] == FORK) {
        # The `eval_safe` function of https://github.com/jeroen/unix provides an alternative 
        # implementation of fork-exec for R. We might use it in the future.
      
        # The Rust process will kill the child so use `estranged` to avoid zombie processes
        # (because this process still has an entry for the child)
        process <- parallel:::mcfork(estranged = TRUE)
        if (!inherits(process, "masterProcess")) {
          # Parent process, so return the pid of the fork and then wait for the next task
          write(paste0(process$pid, TASK), stdout)
          write(TASK, stderr)
          next
        }

        # Child process, so...

        # Remove the FORK flag and the pipe paths from the front of lines
        new_stdout <- lines[2]
        new_stderr <- lines[3]
        lines <- tail(lines, -3)

        # Set stdin to /dev/null to end loop
        stdin <- NULL

        # Replace stdout and stderr with pipes
        stdout <- file(new_stdout, open = "w", raw = TRUE)
        stderr <- file(new_stderr, open = "w", raw = TRUE)
      }

      code <- paste0(lines, collapse = "\n")
      compiled <- tryCatch(parse(text=code), error=identity)
      if (inherits(compiled, "simpleError")) {
        error(compiled, "SyntaxError")
      } else {  
        # Default graphics device to avoid window popping up or `Rplot.pdf` polluting
        # local directory. The tempdir check is needed when forking.
        png(tempfile(tmpdir = tempdir(check=TRUE)))
        # Recording must be enabled for recordPlot() to work
        dev.control("enable")

        value <- tryCatch(
          eval(compiled, envir, .GlobalEnv),
          message=info,
          warning=warning,
          error=error,
          interrupt=interrupt
        )
        
        if (!withVisible(value)$visible) {
          value <- NULL
        }

        rec_plot <- recordPlot()
        if (!is.null(rec_plot[[1]])) {
          value <- rec_plot
        }
        
        # Clear the existing device
        dev.off()  

        if (!is.null(value)) {
          assignment <- grepl("^\\s*\\w+\\s*(<-|=)\\s*", tail(lines, 1))
          if (!assignment) write(paste0(encode_value(value), RESULT), stdout)
        }
      }

      write(TASK, stdout)
      write(TASK, stderr)
    },
    interrupt=function(condition){
      saved_task <<- task
    }
  )
}
