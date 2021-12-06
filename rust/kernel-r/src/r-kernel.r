#!/usr/bin/env Rscript

args <- commandArgs(trailingOnly = FALSE)
pattern <- "--file="
match <- grep(pattern, args)
file <- sub(pattern, "", args[match])
# Unescape whitespaces in file paths for macOS
dir <- gsub("\\~\\+\\~", " ", dirname(file))

source(file.path(dir, "r-codec.r"))

res_sep <- "\U0010ABBA"
trans_sep <- "\U0010ACDC"

print <- function(x, ...) write(paste0(encode_value(x), res_sep), stdout())

message <- function(msg, type) write(paste0(encode_message(msg, type), res_sep), stderr())
info <- function(msg) message(msg, "CodeInfo")
warning <- function(msg) message(msg, "CodeWarning")
error <- function(error, type = "RuntimeError") message(error$message, type)

# Default graphics device to avoid window popping up or `Rplot.pdf` polluting
# local directory. Recording must be enabled for print devices.
png(tempfile())
dev.control("enable")

stdin <- file("stdin", "r")
while (TRUE) {
  code <- readLines(stdin, n=1)
  unescaped <- gsub("\\\\n", "\n", code)

  compiled <- tryCatch(parse(text=unescaped), error=identity)
  if (inherits(compiled, "simpleError")) {
    error(compiled, "SyntaxError")
  } else {
    value <- tryCatch(eval(compiled), message=info, warning=warning, error=error)

    if (!withVisible(value)$visible) {
      value <- NULL
    }

    rec_plot <- recordPlot()
    if (!is.null(rec_plot[[1]])) {
      value <- rec_plot
      # Clear the existing device and create a new one
      dev.off()
      png(tempfile())
      dev.control("enable")
    }

    if (!is.null(value)) {
      last_line <- tail(strsplit(unescaped, "\\n")[[1]], n=1)
      assignment <- grepl("^\\s*\\w+\\s*(<-|=)\\s*", last_line)
      if (!assignment) write(paste0(encode_value(value), res_sep), stdout())
    }
  }

  write(trans_sep, stdout())
  write(trans_sep, stderr())
}
