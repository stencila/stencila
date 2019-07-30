context("node")

test_that("node_to_json", {
  expect_equal(node_to_json(as_scalar(TRUE)), "true")
  expect_equal(node_to_json(as_scalar(1)), "1")
  expect_equal(node_to_json(1:5), "[1,2,3,4,5]")
  expect_equal(node_to_json(list(a = as_scalar(1), b = as_scalar(2))), "{\"a\":1,\"b\":2}")
})


test_that("node_from_json", {
  expect_equal(node_from_json("true"), TRUE)
  expect_equal(node_from_json("1"), 1)
  expect_equal(node_from_json("[1,2,3,4,5]"), 1:5)
  expect_equal(node_from_json("{\"a\":1,\"b\":2}"), list(a = 1, b = 2))

  # nolint start
  json <- '{
  "type": "Person",
  "givenNames": ["Jason"],
  "familyNames": ["James"],
  "affiliations": [
    {
      "type": "Organization",
      "name": ["Acme Corp."]
    },
    {
      "type": "Organization",
      "name": ["University of Erewhon"]
    }
  ]
}'
  # nolint end

  jason <- node_from_json(json)
  expect_true(inherits(jason, "Entity"))
  expect_equal(as.character(node_type(jason)), "Person")
  expect_equal(jason$givenNames, "Jason")
  expect_equal(jason$familyNames, "James")
})


test_that("node_to_json + node_from_json", {
  jane <- Person(
    honorificPrefix = "Dr",
    givenNames = "Jane",
    familyNames = c("Jones", "Jamieson"),
    honorificSuffix = "PhD",
  )

  # nolint start
  jane_json <- '{
  "type": "Person",
  "familyNames": ["Jones", "Jamieson"],
  "givenNames": ["Jane"],
  "honorificPrefix": "Dr",
  "honorificSuffix": "PhD"
}'
  # nolint end

  expect_equal(node_to_json(jane, pretty = TRUE), jane_json)
  expect_equal(node_from_json(jane_json), jane)
})
