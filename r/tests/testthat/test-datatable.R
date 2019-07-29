context("datatable")

datatable_colnames <- function(dt) sapply(dt$columns, function(column) column$name)
datatable_coltypes <- function(dt) sapply(dt$columns, function(column) column$schema$items$type)

test_that("datatable_from_dataframe: column types", {
  dt <- datatable_from_dataframe(
    data.frame(
      a = 1:2,
      b = c(TRUE, FALSE),
      c = c("x", "y"),
      d = factor(c("X", "Y"), levels = c("X", "Y", "Z"))
    ),
    stringsAsFactors = FALSE
  )

  expect_equal(length(dt$columns), 4)
  expect_equal(
    datatable_colnames(dt),
    c("a", "b", "c", "d")
  )
  expect_equal(
    datatable_coltypes(dt),
    c("number", "boolean", "string", "string")
  )
  expect_equal(
    dt$columns[[4]]$schema$items$enum,
    c("X", "Y", "Z")
  )
})

test_that("datatable_from_dataframe: mtcars", {
  dt <- datatable_from_dataframe(mtcars)

  expect_equal(length(dt$columns), 11)
  expect_equal(
    datatable_colnames(dt),
    c("mpg", "cyl", "disp", "hp", "drat", "wt", "qsec", "vs", "am", "gear", "carb")
  )
  expect_equal(
    datatable_coltypes(dt),
    c("number", "number", "number", "number", "number", "number", "number", "number", "number", "number", "number")
  )
})

test_that("datatable_from_dataframe: chickwts", {
  dt <- datatable_from_dataframe(chickwts)

  expect_equal(length(dt$columns), 2)
  expect_equal(
    datatable_colnames(dt),
    c("weight", "feed")
  )
  expect_equal(
    datatable_coltypes(dt),
    c("number", "string")
  )
  expect_equal(
    dt$columns[[2]]$schema$items$enum,
    c("casein", "horsebean", "linseed", "meatmeal", "soybean", "sunflower")
  )
})

test_that("datatable_to_dataframe: column types", {
  dt <- Datatable(
    columns = list(
      DatatableColumn(
        name = "a",
        values = c(1, 2)
      ),
      DatatableColumn(
        name = "b",
        values = c(TRUE, FALSE)
      ),
      DatatableColumn(
        name = "c",
        values = c("a", "b")
      ),
      DatatableColumn(
        name = "d",
        schema = DatatableColumnSchema(
          items = list(
            type = "string",
            enum = c("X", "Y", "Z")
          )
        ),
        values = c("X", "Y")
      )
    )
  )
  df <- datatable_to_dataframe(dt)

  expect_equal(ncol(df), 4)
  expect_equal(
    colnames(df),
    c("a", "b", "c", "d")
  )
  expect_true(is.numeric(df$a))
  expect_true(is.logical(df$b))
  expect_true(is.character(df$c))
  expect_false(is.factor(df$c))
  expect_true(is.factor(df$d))
})

test_that("datatable - dataframe round trips", {
  round <- function(df) datatable_to_dataframe(datatable_from_dataframe(df))
  expect_equal(round(iris), iris)
  expect_equal(round(chickwts), chickwts)
})
