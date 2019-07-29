context("types")

test_that("Entity", {
  expect_equal(last_class(Entity()), "Entity")
  expect_true(inherits(Entity(), "Entity"))
})

test_that("Person", {
  jane <- Person(
    honorificPrefix = "Dr",
    givenNames = c("Jane"),
    familyNames = list("Jones", "Jamieson")
  )
  expect_equal(jane$familyNames, c("Jones", "Jamieson"))
})
