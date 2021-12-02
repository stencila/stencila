#!/usr/bin/env Rscript

args <- commandArgs(trailingOnly = FALSE)
pattern <- "--file="
match <- grep(pattern, args)
file <- sub(pattern, "", args[match])
dir <- dirname(file)

source(file.path(dir, "r-codec.r"))

res_sep <- "\U0010ABBA"
trans_sep <- "\U0010ACDC"

print <- function(x, ...) write(paste0(encode_value(x), res_sep), stdout())

message <- function(msg, type) write(paste0(encode_message(msg, type), res_sep), stderr())
info <- function(msg) message(msg, "CodeInfo")
warning <- function(msg) message(msg, "CodeWarning")
error <- function(error, type = "RuntimeError") message(error$message, type)

stdin <- file("stdin", "r")
while (TRUE) {
  code <- readLines(stdin, n=1)
  unescaped <- gsub("\\\\n", "\n", code)

  compiled <- tryCatch(parse(text=unescaped), error=identity)
  if (inherits(compiled, "simpleError")) {
    error(compiled, "SyntaxError")
  } else {
    value <- tryCatch(eval(compiled), message=info, warning=warning, error=error)
    if (!is.null(value)) {
      write(paste0(encode_value(value), res_sep), stdout())
    }
  }

  write(trans_sep, stdout())
  write(trans_sep, stderr())
}
