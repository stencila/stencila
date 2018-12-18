library(bench)
library(stencilaschema)

source(file.path("..", "fixtures", "jsonRpcRequests.R"))

roundtrip <- function(encoder, request) {
  message <- encoder$encode(request)
  encoder$decode(message, JsonRpcRequest)
}

results <- bench::press(
  encoder = c("JsonEncoder", "JsonGzipEncoder"),
  request = names(requests), {
    encoder_instance <- get(encoder)$new()
    request_instance <- requests[[request]]
    bench::mark(
      roundtrip(encoder_instance, request_instance)
    )
  }
)

results
