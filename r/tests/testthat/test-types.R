context("types")

test_that("core typing functions work", {
  expect_equal(last_class(Entity()), "Entity")
  expect_true(inherits(Entity(), "Entity"))
})

test_that("can construct a simple node", {
  jane <- Person(
    honorificPrefix = "Dr",
    givenNames = c("Jane"),
    familyNames = list("Jones", "Jamieson")
  )
  expect_equal(jane$familyNames, c("Jones", "Jamieson"))
})

test_that("arguments to constructor functions are checked", {
  expect_error(
    Datatable(),
    "Datatable\\$columns is required"
  )

  expect_error(
    Datatable(
      columns = list(
        Person()
      )
    ),
    "Datatable\\$columns is type list, expected type Array\\(DatatableColumn\\)"
  )

  expect_error(
    Datatable(
      columns = list(
        DatatableColumn(
          name = "A"
        )
      )
    ),
    "DatatableColumn\\$values is required"
  )

  expect_error(
    Datatable(
      columns = list(
        DatatableColumn(
          name = "A",
          values = matrix()
        )
      )
    ),
    "DatatableColumn\\$values is type matrix, expected type Array\\(Any\\(\\)\\)"
  )

  expect_error(
    Datatable(
      columns = list(
        DatatableColumn(
          name = "A",
          values = 1,
          validator = NumberValidator()
        )
      )
    ),
    "DatatableColumn\\$validator is type NumberValidator, expected type ArrayValidator$"
  )

  expect_equal(
    Datatable(
      columns = list(
        DatatableColumn(
          name = "A",
          values = 1:10,
          validator = ArrayValidator(items = NumberValidator())
        )
      )
    )$columns[[1]]$values,
    1:10
  )
})
