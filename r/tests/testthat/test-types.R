context("types")

test_that("Entity", {
  expect_equal(class(Entity()), c("list", "Entity"))
})

test_that("Person", {
  jane <- Person(
    honorificPrefix = "Dr",
    givenNames = c("Jane"),
    familyNames = list("Jones", "Jamieson")
  )
  expect_equal(jane$familyNames, c("Jones", "Jamieson"))
})
