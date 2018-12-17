context("JsonEncoder")

describe("JsonEncoder", {
  encoder <- JsonEncoder$new()

  it("has a name", {
    expect_equal(encoder$name(), "json")
  })

  it("encodes and decodes requests", {
    source(file.path("..", "fixtures", "requests.R"))
    for (name in names(requests)) {
      request1 <- requests[[name]]
      message <- encoder$encode(request1)
      request2 <- encoder$decode(message, Request)
      expect_equal(request1, request2, name)
    }
  })

  it("encodes and decodes responses", {
    source(file.path("..", "fixtures", "responses.R"))
    for (name in names(responses)) {
      response1 <- responses[[name]]
      message <- encoder$encode(response1)
      response2 <- encoder$decode(message, Response)
      expect_equal(response1, response2, name)
    }
  })
})
