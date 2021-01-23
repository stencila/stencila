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
    "DatatableColumn\\$values is type array, expected type Array\\(Any\\(\\)\\)"
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

test_that("primitives in Array properties are treated as scalars", {
  chunk <- CodeChunk(
    # These should all be scalars
    programmingLanguage = "r",
    text = "plot(1)",
    label = "Figure 1",
    id = "fig1",
    caption = list(
      Heading(
        # So should the strings here inside an array...
        content = list("Figure title"),
        depth = 2
      ),
      Paragraph(
        content = list(
          "A paragraph with some",
          # Including here, inside a nested array...
          Strong(content = list("strong emphasis")),
          "in it."
        )
      )
    )
  )
  expect_true(inherits(chunk$programmingLanguage, "scalar"))
  expect_true(inherits(chunk$text, "scalar"))
  expect_true(inherits(chunk$label, "scalar"))
  expect_true(inherits(chunk$id, "scalar"))

  expect_equal(class(chunk$caption), "list")
  expect_equal(class(chunk$caption[[1]]$depth), c("scalar", "numeric"))
  expect_equal(class(chunk$caption[[1]]$content[[1]]), c("scalar", "character"))
})
