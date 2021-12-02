if (!suppressPackageStartupMessages(require("jsonlite", quietly=TRUE)))
    install.packages("jsonlite")

decode_value <- function(json) {
    jsonlite::fromJSON(json)
}

encode_value <- function(value) {
    jsonlite::toJSON(value)
}

encode_message <- function(message, type) {
  escaped <- gsub('\\"', '\\\\"', message)
  escaped <- gsub('\\n', '\\\\n', escaped)
  paste0('{"type":"CodeError","errorType":"', type, '","errorMessage":"', escaped, '"}')
}
