#!/usr/bin/env Rscript

res_sep <- '\U0010ABBA'
trans_sep <- '\U0010ACDC'

stdin <- file('stdin', 'r')
while (TRUE) {
  code <- readLines(stdin, n=1)
  unescaped <- gsub("\\\\n", "\n", code)

  compiled <- parse(text=unescaped)
  out <- eval(compiled)
  write(res_sep, stdout())
  write(out, stdout())

  write(trans_sep, stdout())
  write(trans_sep, stderr())
}
