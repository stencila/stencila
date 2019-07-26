context("json")

test_that("to_json + from_json", {
  jane <- Person(
    honorificPrefix = "Dr",
    givenNames = "Jane"
  )

  # nolint start
  jane_json <- '{
  "type": "Person",
  "givenNames": ["Jane"],
  "honorificPrefix": "Dr"
}'
  # nolint end

  expect_equal(to_json(jane, pretty = TRUE), jane_json)
  expect_equal(from_json(jane_json), jane)
})
