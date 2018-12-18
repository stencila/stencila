context("encoders")

encoders <- list(
  "json" = JsonEncoder,
  "json+gzip" = JsonGzipEncoder
)
for (name in names(encoders)) {
  describe(name, {
    encoder <- encoders[[name]]$new()

    it("has a name", {
      expect_equal(encoder$name(), name)
    })

    it("encodes and decodes requests", {
      source(file.path("..", "fixtures", "jsonRpcRequests.R"))
      for (name in names(requests)) {
        request1 <- requests[[name]]
        message <- encoder$encode(request1)
        request2 <- encoder$decode(message, JsonRpcRequest)
        expect_equal(request1, request2, name)
      }
    })

    it("encodes and decodes responses", {
      source(file.path("..", "fixtures", "jsonRpcResponses.R"))
      for (name in names(responses)) {
        response1 <- responses[[name]]
        message <- encoder$encode(response1)
        response2 <- encoder$decode(message, JsonRpcResponse)
        expect_equal(response1, response2, name)
      }
    })
  })
}
