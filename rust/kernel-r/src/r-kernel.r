#!/usr/bin/env Rscript

# Ensure that required packages are attached and installed
requires <- function () {
  # On Mac and Linux require `parallel` package for forking
  # This is a base package (see `rownames(installed.packages(priority="base"))`)
  # so we don't try to install it as with other packages
  if (.Platform$OS.type == "unix") library(parallel)

  pkgs <- c("jsonlite", "base64enc")

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

# Use easier-to-type flags during development manual testing
if (isatty(stdin())) {
  READY <- "READY"
  RESULT <- "RESULT"
  TASK <- "TASK"
  FORK <- "FORK"
  NEWLINE <- "NEWLINE"
  EXIT <- "EXIT"
} else {
  READY <- "\U0010ACDC"
  RESULT <-  "\U0010CB40"
  TASK <- "\U0010ABBA"
  FORK <- "\U0010DE70"
  NEWLINE <- "\U0010B522"
  EXIT <- "\U0010CC00"
}

stdin <- file("stdin", "r")
stdout <- stdout()
stderr <- stderr()

# Functions to encode messages as `CodeMessage`
message <- function(msg, type) write(paste0(encode_message(msg, type), RESULT), stderr)
info <- function(msg) message(msg, "Info")
warning <- function(msg) message(msg, "Warning")
error <- function(error, type = "RuntimeError") message(error$message, type)
interrupt <- function(condition, type = "Interrupt") message("Code execution was interrupted", type)

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
      # line is read. So, we have to save the task in case interrupt was inadvertently called during
      # readline and then "replay" it on the next loop.
      if (is.null(saved_task)) {
        task <<- readLines(stdin, n=1)
      } else {
        task <<- saved_task
      }
      saved_task <<- NULL

      # If there is no task from `readLines` it means `stdin` was closed, so exit gracefully
      if (length(task) == 0) quit(save = "no")

      lines <- strsplit(task, NEWLINE, fixed = TRUE)[[1]]

      if (lines[1] == EXIT) {
        quit(save="no")
      }

      should_exec <- TRUE
      should_exit <- FALSE
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

        # Remove the FORK flag and the pipe paths from the front of lines
        new_stdin <- lines[2]
        new_stdout <- lines[3]
        new_stderr <- lines[4]
        lines <- tail(lines, -4)

        if (nzchar(new_stdin) > 0) {
          # Replace stdin with pipe and do not execute code (which should be empty)
          stdin <- file(new_stdin, open = "r", raw = TRUE)
          should_exec <- FALSE   
        } else {
          # If there is no new stdin then set stdin to /dev/null to avoid getting more input
          # and exit at end of loop
          stdin <- NULL
          should_exit <- TRUE
        }

        # Replace stdout and stderr with pipes
        # These will normally be not NA except when using the FORK flag during manual testing
        if (!is.na(new_stdout) && !is.na(new_stderr)) {
          stdout <- file(new_stdout, open = "w", raw = TRUE)
          stderr <- file(new_stderr, open = "w", raw = TRUE)
        }
      }

      if (should_exec) {
        code <- paste0(lines, collapse = "\n")
        compiled <- tryCatch(parse(text=code), error=identity)
        if (inherits(compiled, "simpleError")) {
          error(compiled, "SyntaxError")
        } else {
          # Default graphics device to avoid window popping up or `Rplot.pdf` polluting
          # local directory. 
          # `CairoPNG` is preferred instead of `png` to avoid "a forked child should not open a graphics device"
          # which arises because X11 can not be used in a forked environment.
          # The tempdir `check` is needed when forking.
          file <- tempfile(tmpdir = tempdir(check=TRUE))
          tryCatch(
            Cairo::CairoPNG(file),
            error = function(cond) png(file)
          )
          # Recording must be enabled for recordPlot() to work
          dev.control("enable")

          # Capture output to stdout so we can add a terminating flag
          #output <- textConnection("out", "w", local = TRUE)
          #sink(output, type = "output")

          value <- tryCatch(
            eval(compiled, envir, .GlobalEnv),
            message=info,
            warning=warning,
            error=error,
            interrupt=interrupt
          )

          # Get any output and reset the sink
          #output_text <- textConnectionValue(output)
          #sink(type = "output")
          #close(output)
          #if (nzchar(output_text)) {
          #  write(paste0(output_text, RESULT), stdout)
          #}
          
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
            if (!blank && !comment && !assignment) write(paste0(encode_value(value), RESULT), stdout)
          }
        }

        write(TASK, stdout)
        write(TASK, stderr)
      }

      if (should_exit) {
        quit(save="no")
      }
    },
    interrupt=function(condition){
      saved_task <<- task
    }
  )
}
