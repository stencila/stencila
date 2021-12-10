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
TRANS <- "\U0010ABBA"
FORK <- "\U0010DE70"

stdin <- file("stdin", "r")
stdout <- stdout()
stderr <- stderr()

# Monkey patch `print` to encode individual objects
print <- function(x, ...) write(paste0(encode_value(x), RESULT), stdout)

message <- function(msg, type) write(paste0(encode_message(msg, type), RESULT), stderr)
info <- function(msg) message(msg, "CodeInfo")
warning <- function(msg) message(msg, "CodeWarning")
error <- function(error, type = "RuntimeError") message(error$message, type)

write(READY, stdout)
write(READY, stderr)

while (!is.null(stdin)) {
  code <- readLines(stdin, n=1)

  if (endsWith(code, FORK)) {
    # The `eval_safe` function of https://github.com/jeroen/unix provides an alternative 
    # implementation of fork-exec for R. We might use it in the future.
  
    process <- parallel:::mcfork()
    if (!inherits(process, "masterProcess")) {
      # Parent process so just go to the next line
      next
    }

    # Child process, so...

    # Separate code and paths of FIFO pipes to replace stdout and stderr
    code <- substr(code, 1, nchar(code) - nchar(FORK))
    parts <- strsplit(code, "\\|")[[1]]
    code <- paste0(head(parts, n=length(parts) - 1), collapse = "|")
    pipes <- strsplit(tail(parts, n = 1), ";")[[1]]

    # Set stdin to /dev/null to end loop
    stdin <- NULL

    # Replace stdout and stderr with pipes
    stdout <- file(pipes[1], open = "w", raw = TRUE)
    stderr <- file(pipes[2], open = "w", raw = TRUE)
  }

  unescaped <- gsub("\\\\n", "\n", code)

  compiled <- tryCatch(parse(text=unescaped), error=identity)
  if (inherits(compiled, "simpleError")) {
    error(compiled, "SyntaxError")
  } else {  
    # Default graphics device to avoid window popping up or `Rplot.pdf` polluting
    # local directory. The tempdir check is needed when forking.
    png(tempfile(tmpdir = tempdir(check=TRUE)))
    # Recording must be enabled for recordPlot() to work
    dev.control("enable")

    value <- tryCatch(eval(compiled), message=info, warning=warning, error=error)
    
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
      last_line <- tail(strsplit(unescaped, "\\n")[[1]], n=1)
      assignment <- grepl("^\\s*\\w+\\s*(<-|=)\\s*", last_line)
      if (!assignment) write(paste0(encode_value(value), RESULT), stdout)
    }
  }

  write(TRANS, stdout)
  write(TRANS, stderr)
}
